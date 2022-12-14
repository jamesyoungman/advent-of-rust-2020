extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate lazy_static;
extern crate ndarray;
extern crate regex;

use ndarray::prelude::*;
use ndarray::s;
use ndarray::Zip;
use regex::Regex;
use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::io;
use std::io::Read;
use std::num::ParseIntError;
use std::str::FromStr;

lazy_static! {
    static ref TILE_TITLE_RE: Regex = Regex::new("^Tile ([0-9]*):$").unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Direction {
    N,
    E,
    S,
    W,
}

impl Direction {
    fn all() -> [Direction; 4] {
        [Direction::N, Direction::E, Direction::S, Direction::W]
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Direction::N => "N",
            Direction::E => "E",
            Direction::S => "S",
            Direction::W => "W",
        })
    }
}

fn opposite_direction(d: &Direction) -> Direction {
    match d {
        Direction::N => Direction::S,
        Direction::E => Direction::W,
        Direction::S => Direction::N,
        Direction::W => Direction::E,
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TileId {
    val: i32,
}

impl From<i32> for TileId {
    fn from(n: i32) -> Self {
        TileId { val: n }
    }
}

impl From<&i32> for TileId {
    fn from(n: &i32) -> Self {
        TileId { val: *n }
    }
}

impl FromStr for TileId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse() {
            Err(e) => Err(e),
            Ok(n) => Ok(TileId { val: n }),
        }
    }
}

impl fmt::Display for TileId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

#[derive(Debug, Copy, Clone)]
enum Rotation {
    Zero,
    One,
    Two,
    Three,
}

#[derive(Copy, Clone)]
struct Manipulation {
    rot: Rotation,
    flip: bool,
}

impl Manipulation {
    fn new(n: i32) -> Manipulation {
        Manipulation {
            rot: match n & 0x03 {
                0 => Rotation::Zero,
                1 => Rotation::One,
                2 => Rotation::Two,
                3 => Rotation::Three,
                _ => panic!("implossible"),
            },
            flip: match n & 0x04 {
                0 => false,
                4 => true,
                _ => panic!("implossible"),
            },
        }
    }

    fn noop() -> Manipulation {
        Manipulation {
            rot: Rotation::Zero,
            flip: false,
        }
    }

    fn all() -> Vec<Manipulation> {
        (0..8).map(Manipulation::new).collect()
    }

    fn as_string(&self) -> String {
        format!(
            "R{}F{}",
            match self.rot {
                Rotation::Zero => 0,
                Rotation::One => 1,
                Rotation::Two => 2,
                Rotation::Three => 3,
            },
            if self.flip { "Y" } else { "N" }
        )
    }

    fn do_rot(&self, tiledata: Array2<u8>) -> Array2<u8> {
        // rotations are counter-clockwise.
        match self.rot {
            Rotation::Zero => tiledata,
            Rotation::One => tiledata.slice(s![.., ..;-1]).reversed_axes().into_owned(),
            Rotation::Two => tiledata.slice(s![..;-1, ..;-1]).into_owned(),
            Rotation::Three => tiledata.slice(s![..;-1, ..]).reversed_axes().into_owned(),
        }
    }

    fn do_flip(&self, tiledata: Array2<u8>) -> Array2<u8> {
        if self.flip {
            tiledata.slice(s![.., ..;-1]).into_owned()
        } else {
            tiledata
        }
    }

    fn on(&self, m: &Array2<u8>) -> Array2<u8> {
        self.do_rot(self.do_flip(m.to_owned()))
    }
}

impl FromStr for Manipulation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut it = s.chars();
        if let Some(ch) = it.next() {
            if ch != 'R' {
                return Err("must begin with R".to_string());
            }
        }
        let rot = match it.next() {
            None => {
                return Err("string is too short".to_string());
            }
            Some('0') => Rotation::Zero,
            Some('1') => Rotation::One,
            Some('2') => Rotation::Two,
            Some('3') => Rotation::Three,
            Some(ch) => {
                return Err(format!("invalid rotation {}", ch));
            }
        };
        if let Some(ch) = it.next() {
            if ch != 'F' {
                return Err("must have F as the third character".to_string());
            }
        }
        let flip = match it.next() {
            Some('Y') => true,
            Some('N') => false,
            _ => {
                return Err("flip must be Y or N".to_string());
            }
        };
        match it.next() {
            None => Ok(Manipulation { rot, flip }),
            _ => Err("trailing garbage at the end of the string".to_string()),
        }
    }
}

impl fmt::Display for Manipulation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.as_string())
    }
}

impl fmt::Debug for Manipulation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.as_string())
    }
}

#[derive(Debug)]
struct Tile {
    id: TileId,
    d: Array2<u8>,
}

impl Tile {
    fn from_string(s: &str) -> Result<Tile, String> {
        let lines: Vec<String> = s.split('\n').map(str::to_string).collect();
        if lines.is_empty() {
            return Err("Tiles must not be empty".to_string());
        }
        let id: TileId = match TILE_TITLE_RE.captures(&lines[0]) {
            Some(caps) => match caps[1].parse() {
                Ok(n) => n,
                Err(e) => {
                    return Err(format!(
                        "failed to parse '{}' as an integer: {}",
                        &caps[1], e
                    ));
                }
            },
            None => {
                return Err(format!("tile is missing a title:\n{}", s));
            }
        };
        log::debug!("tile id is {}", id);
        let width = lines[1].len();
        let height = lines.len() - 1;
        if width != height {
            return Err(format!(
                "Tiles should be square but this has {} rows, {} columns: {:?}",
                height, width, lines
            ));
        }
        let tiledata = s[lines[0].len() + 1..].chars().collect::<Vec<char>>();
        let d = Array::from_shape_fn((height, width), |(r, c)| {
            decode_ascii_tile(&id, r, c, &width, &tiledata)
        });
        if d.iter().filter(|x| **x == 2).count() > 0 {
            log::debug!("bad tile data:\n{}", &s[width + 1..]);
            return Err("tile data contained unexpected characters".to_string());
        }
        Ok(Tile { id, d })
    }

    fn manipulated(&self, how: &Manipulation) -> Array2<u8> {
        how.on(&self.d)
    }

    fn interior(&self, how: &Manipulation) -> Array2<u8> {
        self.manipulated(how)
            .slice(s![1..self.d.nrows() - 1, 1..self.d.ncols() - 1])
            .into_owned()
    }
}

#[derive(Debug, Clone)]
struct TileIndexEntry {
    tile_id: TileId,
    manipulation: Manipulation,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct EdgePattern {
    bits: i32,
}

impl EdgePattern {
    fn from_edge(edge: &ArrayView1<u8>) -> EdgePattern {
        EdgePattern {
            bits: edge.iter().fold(0, |bits, elem| {
                let bit = match elem {
                    0 => 0,
                    1 => 1,
                    _ => panic!("matrix should be 0/1 only"),
                };
                (bits << 1) | bit
            }),
        }
    }

    fn from_matrix(d: &Direction, m: &ArrayView2<u8>) -> EdgePattern {
        EdgePattern::from_edge(&m.slice(&match d {
            Direction::N => s![0, ..].to_owned(),
            Direction::E => s![.., m.ncols() - 1].to_owned(),
            Direction::S => s![m.nrows() - 1, ..].to_owned(),
            Direction::W => s![.., 0].to_owned(),
        }))
    }
}

impl fmt::Display for EdgePattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.bits, f)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct EdgeKey {
    direction: Direction,
    pattern: EdgePattern,
}

impl EdgeKey {
    fn from_matrix(d: &Direction, m: &ArrayView2<u8>) -> EdgeKey {
        EdgeKey {
            direction: *d,
            pattern: EdgePattern::from_matrix(d, m),
        }
    }

    fn opposing(&self) -> EdgeKey {
        EdgeKey {
            direction: opposite_direction(&self.direction),
            pattern: self.pattern,
        }
    }
}

impl fmt::Display for EdgeKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.direction, self.pattern)
    }
}

#[derive(Debug, Clone, Copy)]
struct ExposedEdge {
    edge_pattern: EdgePattern,
    pos: Position,
    direction: Direction,
}

#[derive(Debug)]
struct TileLocationSolution {
    tile_to_position: HashMap<TileId, (Position, Manipulation)>,
    position_to_tile: HashMap<Position, TileId>,
    exposed_edges: Vec<ExposedEdge>,
}

impl TileLocationSolution {
    fn new() -> TileLocationSolution {
        TileLocationSolution {
            tile_to_position: HashMap::new(),
            position_to_tile: HashMap::new(),
            exposed_edges: Vec::new(),
        }
    }

    fn occupied(&self, pos: &Position) -> bool {
        self.position_to_tile.contains_key(pos)
    }

    fn place_tile(&mut self, tile: &Tile, how: &Manipulation, pos: &Position) {
        assert!(!self.tile_to_position.contains_key(&tile.id));
        assert!(!self.position_to_tile.contains_key(pos));
        self.tile_to_position.insert(tile.id, (*pos, *how));
        self.position_to_tile.insert(*pos, tile.id);

        // Remove existing exposed edges which are now covered.
        // Borrowing rules prevent the use of .retain() in its most
        // obvious form.
        self.exposed_edges = self
            .exposed_edges
            .iter()
            .filter(|exposure| {
                let neighbour = get_neighbour(&exposure.pos, exposure.direction);
                !self.occupied(&neighbour)
            })
            .copied()
            .collect();
        // Add new exposures for any edges of the new piece that are not touching
        // already-placed pieces.
        let tile_bits = tile.manipulated(how);
        for d in Direction::all().iter() {
            // Get the location of the possible neighbour in this direction.
            let n = get_neighbour(pos, *d);
            // If that position is not already occupied, this edge is exposed.
            if !self.occupied(&n) {
                self.exposed_edges.push(ExposedEdge {
                    edge_pattern: EdgePattern::from_matrix(d, &tile_bits.view()),
                    pos: *pos,
                    direction: *d,
                });
            }
        }
    }

    fn get_position_of_tile(&self, tile_id: &TileId) -> Option<&(Position, Manipulation)> {
        self.tile_to_position.get(tile_id)
    }

    fn get_tile_at_position(&self, pos: &Position) -> Option<(TileId, Manipulation)> {
        if let Some(tid) = self.position_to_tile.get(pos) {
            if let Some((_, manip)) = self.get_position_of_tile(tid) {
                Some((*tid, *manip))
            } else {
                panic!("TileLocationSolution inconsistnecy");
            }
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.tile_to_position.len()
    }
}

fn decode_ascii_tile(id: &TileId, r: usize, c: usize, width: &usize, s: &[char]) -> u8 {
    let pos: usize = (width + 1) * r + c;
    match s[pos] {
        '#' => 1,
        '.' => 0,
        _ => {
            log::debug!("decode_ascii_tile: unexpected character {} at r={}, c={} in defintiion for tile {}",
		     s[pos], r, c, id);
            2 //signal an error.
        }
    }
}

fn self_test() -> Result<(), String> {
    let case1 = "Tile 1:
.###.
#...#
#....
#....
#....

Tile 2:
.#..#
#....
.....
....#
..#..

Tile 3:
#....
.....
#....
#...#
.#.##

Tile 4:
..#..
.....
....#
#....
#####
"
    .to_string();
    let tiles1 = read_tiles(&case1);

    // check 4, R1FN
    let t4 = &tiles1.get(&TileId { val: 4 }).unwrap().d;
    let t4_r1fn = Manipulation {
        rot: Rotation::One,
        flip: false,
    }
    .on(t4);
    assert_eq!(
        t4_r1fn,
        arr2(&[
            [0, 0, 1, 0, 1],
            [0, 0, 0, 0, 1],
            [1, 0, 0, 0, 1],
            [0, 0, 0, 0, 1],
            [0, 0, 0, 1, 1]
        ])
    );

    // check 4, R1FY
    let t4_r1fy = Manipulation {
        rot: Rotation::One,
        flip: true,
    }
    .on(t4);
    log::debug!("t4={}", t4);
    log::debug!("t4_r1fy={}", t4_r1fy);
    assert_eq!(
        t4_r1fy,
        arr2(&[
            [0, 0, 0, 1, 1],
            [0, 0, 0, 0, 1],
            [1, 0, 0, 0, 1],
            [0, 0, 0, 0, 1],
            [0, 0, 1, 0, 1]
        ])
    );

    // Check 1<->2 match
    assert!(edge_match(
        &tiles1.get(&TileId { val: 1 }).unwrap().d.view(),
        &Direction::E,
        &tiles1.get(&TileId { val: 2 }).unwrap().d.view()
    ));
    assert!(edge_match(
        &tiles1.get(&TileId { val: 2 }).unwrap().d.view(),
        &Direction::W,
        &tiles1.get(&TileId { val: 1 }).unwrap().d.view()
    ));

    // Check 1<->3 match
    assert!(edge_match(
        &tiles1.get(&TileId { val: 1 }).unwrap().d.view(),
        &Direction::S,
        &tiles1.get(&TileId { val: 3 }).unwrap().d.view()
    ));
    assert!(edge_match(
        &tiles1.get(&TileId { val: 3 }).unwrap().d.view(),
        &Direction::N,
        &tiles1.get(&TileId { val: 1 }).unwrap().d.view()
    ));

    // Check 2<->4 match
    assert!(edge_match(
        &tiles1.get(&TileId { val: 2 }).unwrap().d.view(),
        &Direction::S,
        &tiles1.get(&TileId { val: 4 }).unwrap().d.view()
    ));
    assert!(edge_match(
        &tiles1.get(&TileId { val: 4 }).unwrap().d.view(),
        &Direction::N,
        &tiles1.get(&TileId { val: 2 }).unwrap().d.view()
    ));

    // Check 3<->4 match
    assert!(edge_match(
        &tiles1.get(&TileId { val: 3 }).unwrap().d.view(),
        &Direction::E,
        &tiles1.get(&TileId { val: 4 }).unwrap().d.view()
    ));
    assert!(edge_match(
        &tiles1.get(&TileId { val: 4 }).unwrap().d.view(),
        &Direction::W,
        &tiles1.get(&TileId { val: 3 }).unwrap().d.view()
    ));

    let ix1 = make_tile_index(&tiles1);
    log::debug!("self_test: tile index is: {:?}", ix1);
    let initial_manip = &Manipulation::noop();
    let sol1 = solve1(&tiles1, &ix1, initial_manip);
    log::debug!("self_test: solution is {:?}", sol1);

    Ok(())
}

fn read_tiles(s: &str) -> HashMap<TileId, Tile> {
    let r: Result<HashMap<TileId, Tile>, String> = s
        .trim()
        .split("\n\n")
        .map(|s| s.trim())
        .map(|s| match Tile::from_string(s) {
            Ok(t) => Ok((t.id, t)),
            Err(e) => Err(e),
        })
        .collect();
    r.unwrap_or_else(|_| panic!("tiles are not in the expected format"))
}

fn get_edge_keys(m: &ArrayView2<u8>) -> [EdgeKey; 4] {
    [
        EdgeKey::from_matrix(&Direction::N, m),
        EdgeKey::from_matrix(&Direction::E, m),
        EdgeKey::from_matrix(&Direction::S, m),
        EdgeKey::from_matrix(&Direction::W, m),
    ]
}

fn make_tile_index(tiles: &HashMap<TileId, Tile>) -> HashMap<EdgePattern, Vec<TileIndexEntry>> {
    let mut result: HashMap<EdgePattern, Vec<TileIndexEntry>> = HashMap::new();
    for t in tiles.values() {
        let variants: Vec<(Manipulation, TileId, Array2<u8>)> = Manipulation::all()
            .iter()
            .map(|manip| (*manip, t.id, manip.on(&t.d)))
            .collect();
        let mut seen_keys: HashMap<String, Vec<(TileId, Manipulation)>> = HashMap::new();
        for (manip, tid, v) in &variants {
            let mk: String = v.iter().map(|x| x.to_string()).collect();
            seen_keys
                .entry(mk.clone())
                .or_insert_with(Vec::new)
                .push((*tid, *manip));
            assert_eq!(seen_keys.get(&mk).unwrap().len(), 1);

            for edge_key in get_edge_keys(&v.view()).iter() {
                result
                    .entry(edge_key.pattern)
                    .or_insert_with(Vec::new)
                    .push(TileIndexEntry {
                        tile_id: *tid,
                        manipulation: *manip,
                    });
            }
        }
    }
    result
}

fn get_neighbour(pos: &Position, edge: Direction) -> Position {
    match edge {
        Direction::N => Position {
            y: pos.y + 1,
            ..*pos
        },
        Direction::E => Position {
            x: pos.x + 1,
            ..*pos
        },
        Direction::S => Position {
            y: pos.y - 1,
            ..*pos
        },
        Direction::W => Position {
            x: pos.x - 1,
            ..*pos
        },
    }
}

fn place(
    tile_id: &TileId,
    how: &Manipulation,
    pos: &Position,
    tiles: &HashMap<TileId, Tile>,
    solution: &mut TileLocationSolution,
    todo: &mut HashSet<TileId>,
) {
    let tile: &Tile = match tiles.get(tile_id) {
        Some(t) => t,
        _ => {
            panic!(
                "Tile to be placed ({}) is not in the main tile index",
                tile_id
            );
        }
    };
    if let Some(existing) = solution.get_position_of_tile(tile_id) {
        panic!(
            "place() was called to place tile id {} but this was already placed at {:?}",
            tile_id, existing
        );
    }
    // This assertion fires if the tile being placed was absent from the todo set (i.e.
    // when `positions` and `todo` have become out-of-sync).
    assert!(todo.remove(tile_id));
    solution.place_tile(tile, how, pos);
}

fn edge_match(
    candidate: &ArrayView2<u8>,
    neighbour_direction: &Direction,
    neighbour: &ArrayView2<u8>,
) -> bool {
    let candidate_edge_key = EdgeKey::from_matrix(neighbour_direction, candidate);
    let neighbour_edge_key =
        EdgeKey::from_matrix(&opposite_direction(neighbour_direction), neighbour);
    let opposing = neighbour_edge_key.opposing();
    let result = opposing == candidate_edge_key;
    let desc = if result { "match" } else { "no match" };
    log::debug!(
        "edge_match: {} side: checking {} against {}: {}",
        neighbour_direction,
        opposing,
        candidate_edge_key,
        desc
    );
    result
}

fn candidate_fits_neighbours(
    cand: &TileIndexEntry,
    proposed_pos: &Position,
    tiles: &HashMap<TileId, Tile>,
    solution: &TileLocationSolution,
) -> bool {
    log::debug!(
        "Trying to find out whether tile {} ({}) fits at {}",
        cand.tile_id,
        cand.manipulation,
        proposed_pos
    );
    assert!(!solution.occupied(proposed_pos));
    let candidate_tile: &Tile = tiles.get(&cand.tile_id).expect("candidate not in tile map");
    for neighbour_direction in Direction::all().iter() {
        log::debug!(
            "checking for a neighbour of {} in direction: {}",
            proposed_pos,
            neighbour_direction
        );
        let neighbour_pos = get_neighbour(proposed_pos, *neighbour_direction);
        let (neighbour_id, neighbour_manipulation) =
            match solution.get_tile_at_position(&neighbour_pos) {
                None => {
                    log::debug!(
                    "No need to check {} side of {} (i.e. location {}) as there is nothing there",
                    neighbour_direction,
                    proposed_pos,
                    neighbour_pos
                );
                    continue;
                }
                Some(x) => x,
            };
        log::debug!(
            "Neighbour at {} is tile {} ({})",
            neighbour_pos,
            neighbour_id,
            neighbour_manipulation
        );
        let neighbour: &Tile = tiles.get(&neighbour_id).expect("missing neighbour");
        if !edge_match(
            &candidate_tile.manipulated(&cand.manipulation).view(),
            neighbour_direction,
            &neighbour.manipulated(&neighbour_manipulation).view(),
        ) {
            log::debug!("candidate_fits_neighbours: no, tile {} cannot be placed at {} becauise it does not match its neighbour {} at {}",
		     cand.tile_id, proposed_pos, neighbour_id, neighbour_pos);
            return false;
        } else {
            log::debug!("OK so far");
        }
    }
    log::debug!(
        "candidate_fits_neighbours: result is yes, tile {} can be placed at {}",
        cand.tile_id,
        proposed_pos
    );
    true
}

fn get_candidates(
    tiles: &HashMap<TileId, Tile>,
    ix: &HashMap<EdgePattern, Vec<TileIndexEntry>>,
    solution: &TileLocationSolution,
    exposed_edge: &ExposedEdge,
) -> HashMap<TileId, Vec<Manipulation>> {
    let mut result: HashMap<TileId, Vec<Manipulation>> = HashMap::new();
    let empty_pos = get_neighbour(&exposed_edge.pos, exposed_edge.direction);
    if let Some((tid, manip)) = solution.get_tile_at_position(&empty_pos) {
        panic!(
            "unexpectedly, position {} is not empty; it contains tile {} with manipulation {}",
            empty_pos, tid, manip
        );
    };
    for cand in ix
        .get(&exposed_edge.edge_pattern)
        .expect("edge missing from index")
    {
        if solution.get_position_of_tile(&cand.tile_id).is_some() {
            // TODO: we could probably avoid the need for this
            // loop by changing data structure such that we
            // consider only relevant tiles.
            continue;
        }
        if candidate_fits_neighbours(cand, &empty_pos, tiles, solution) {
            log::debug!("candidate {:?} would fit at {}", cand, empty_pos);
            result
                .entry(cand.tile_id)
                .or_insert_with(Vec::new)
                .push(cand.manipulation);
        }
    }
    result
}

fn solve1x(
    tiles: &HashMap<TileId, Tile>,
    ix: &HashMap<EdgePattern, Vec<TileIndexEntry>>,
    solution: &mut TileLocationSolution,
    todo: &mut HashSet<TileId>,
) {
    if solution.len() == tiles.len() {
        panic!("solve1x was called with all tiles already placed");
    }

    for (t, (pos, manip)) in solution.tile_to_position.iter() {
        log::debug!("solve1x: tile {} is at {} ({})", t, pos, manip);
    }

    for exposed_edge in solution.exposed_edges.iter() {
        let pos = get_neighbour(&exposed_edge.pos, exposed_edge.direction);
        let candidates: HashMap<TileId, Vec<Manipulation>> =
            get_candidates(tiles, ix, solution, exposed_edge);
        log::debug!(
            "There are {} candidate tiles for occupation of {}",
            candidates.len(),
            pos
        );
        if candidates.len() > 1 {
            log::debug!("Since there's more than one option for {} we will defer filling that spot for now.", pos);
        } else {
            for (tile_id, manipulations) in candidates.iter() {
                log::debug!(
                    "Tile {} will fit at {} in {} different ways",
                    tile_id,
                    pos,
                    manipulations.len()
                );
                match manipulations.as_slice() {
                    [manip] => {
                        place(tile_id, manip, &pos, tiles, solution, todo);
                        return;
                    }
                    [] => (),
                    _ => {
                        log::debug!("Since there's more than one way to fit {} into {} we will defer filling that spot for now.", tile_id, pos);
                    }
                }
            }
        }
    }
    panic!("solve1x: failed to make any progress");
}

fn solve1(
    tiles: &HashMap<TileId, Tile>,
    ix: &HashMap<EdgePattern, Vec<TileIndexEntry>>,
    initial_manip: &Manipulation,
) -> TileLocationSolution {
    let mut todo: HashSet<TileId> = tiles.keys().copied().collect();
    let initial = match tiles.keys().min() {
        Some(n) => n,
        None => {
            // No tiles, so nothing to do.
            return TileLocationSolution::new();
        }
    };
    log::debug!(
        "Placing intitial tile {} at (0,0) with manipulation {}",
        initial,
        initial_manip
    );
    let mut solution = TileLocationSolution::new();
    place(
        initial,
        initial_manip,
        &Position { x: 0, y: 0 },
        tiles,
        &mut solution,
        &mut todo,
    );
    while !todo.is_empty() {
        log::debug!("solve1: {}/{} tiles left to place", todo.len(), tiles.len());
        let before = todo.len();
        solve1x(tiles, ix, &mut solution, &mut todo);
        let after = todo.len();
        if after >= before {
            panic!("solve1: no progress was made in call to solve1x.");
        }
    }
    log::debug!("solve1: all {} tiles are in place.", tiles.len());
    solution
}

fn min_and_max<T>(things: T) -> (i32, i32)
where
    T: IntoIterator<Item = i32>,
{
    let initial = (0_i32, 0_i32);
    things
        .into_iter()
        .fold(initial, |acc, x| (cmp::min(acc.0, x), cmp::max(acc.1, x)))
}

fn extrema(solution: &TileLocationSolution) -> (i32, i32, i32, i32) {
    let (minx, maxx) = min_and_max(solution.position_to_tile.keys().map(|p| p.x));
    let (miny, maxy) = min_and_max(solution.position_to_tile.keys().map(|p| p.y));
    (minx, maxx, miny, maxy)
}

fn corners(solution: &TileLocationSolution) -> [Position; 4] {
    let (minx, maxx, miny, maxy) = extrema(solution);
    let result = [
        Position { x: minx, y: miny },
        Position { x: minx, y: maxy },
        Position { x: maxx, y: miny },
        Position { x: maxx, y: maxy },
    ];
    for pos in &result {
        match solution.position_to_tile.get(pos) {
            None => {
                panic!("solution is not rectangular; {} is not occupied", pos);
            }
            Some(_) => (),
        }
    }
    result
}

fn corner_product(solution: &TileLocationSolution) -> i64 {
    corners(solution)
        .iter()
        .map(|pos| solution.position_to_tile.get(pos).unwrap().val as i64)
        .product()
}

fn solution_as_string(solution: &TileLocationSolution) -> String {
    let column_width = 1 + cmp::max(
        4, // width of manipulation representation
        format!("{}", solution.tile_to_position.keys().max().unwrap()).len(),
    );
    let mut result = String::new();
    let (minx, maxx, miny, maxy) = extrema(solution);
    for y in miny..=maxy {
        let mut id_row = String::new();
        let mut manip_row = String::new();
        for x in minx..=maxx {
            let pos = Position { x, y };
            let (t, m) = match solution.position_to_tile.get(&pos) {
                Some(tid) => {
                    let manip = solution.tile_to_position.get(tid).unwrap().1;
                    (format!("{}", tid), format!("{}", manip))
                }
                None => ("????".to_string(), "????".to_string()),
            };
            id_row.push_str(&format!("{:width$}", t, width = column_width));
            manip_row.push_str(&format!("{:width$}", m, width = column_width));
        }
        result.push_str(&format!("{}\n{}\n\n", id_row, manip_row));
    }
    result
}

fn part1(tiles: &HashMap<TileId, Tile>) -> Result<TileLocationSolution, String> {
    let ix = make_tile_index(tiles);
    log::debug!("part1: tile index is: {:?}", ix);
    // For convenience in debugging we choose an initial manipulation
    // that makes the result come out lookikng like the sample in the
    // question.
    let initial_manip = &Manipulation {
        rot: Rotation::Zero,
        flip: true,
    };
    let sol = solve1(tiles, &ix, initial_manip);
    println!("Part 1: Solution is:\n{}", solution_as_string(&sol));
    println!("Part 1: corner product is {}", corner_product(&sol));
    Ok(sol)
}

fn interior_tile_at(
    pos: &Position,
    tiles: &HashMap<TileId, Tile>,
    solution: &TileLocationSolution,
) -> Array2<u8> {
    let tile_id = solution.position_to_tile.get(pos).expect("missing tile");
    let (_, manip) = solution
        .tile_to_position
        .get(tile_id)
        .expect("inconsistent solution");
    tiles.get(tile_id).expect("unknown tile").interior(manip)
}

fn assemble_big_bitmap(
    tiles: &HashMap<TileId, Tile>,
    solution: &TileLocationSolution,
) -> Array2<u8> {
    let (minx, maxx, miny, maxy) = extrema(solution);
    let (tile_height, tile_width): (i32, i32) = {
        let some_tile = tiles
            .values()
            .next()
            .expect("input contains no tiles")
            .interior(&Manipulation::noop());
        (some_tile.nrows() as i32, some_tile.ncols() as i32)
    };
    let shape = (
        (tile_height * (maxy - miny + 1)) as usize,
        (tile_width * (maxx - minx + 1)) as usize,
    );
    // We use 2 as a marker value to determine whether we have
    // correctly set all the elements in the output array from the
    // data in the solution.
    let mut result: Array2<u8> = Array::from_elem(shape, 2);
    for (i, y) in (miny..=maxy).rev().enumerate() {
        let ytop: i32 = i as i32 * tile_height;
        for (j, x) in (minx..=maxx).enumerate() {
            let xleft = j as i32 * tile_width;
            let b = interior_tile_at(&Position { x, y }, tiles, solution);
            result
                .slice_mut(s![ytop..ytop + tile_height, xleft..xleft + tile_width])
                .assign(&b);
        }
    }
    match result.iter().max() {
        None => panic!("output is empty"),
        Some(0) | Some(1) => result,
        Some(2) => panic!("some parts of the output array have not been update with the solution"),
        _ => panic!("some parts of the output array have not been initialised"),
    }
}

fn render_bitmap(b: &Array2<u8>) -> String {
    let (height, width) = (b.shape()[0], b.shape()[1]);
    let mut result = String::with_capacity(b.len() + height);
    for (i, elem) in b.iter().enumerate() {
        result.push(match elem {
            0 => '.',
            1 => '#',
            _ => '?',
        });
        if (i + 1) % 8 == 0 {
            result.push(' ');
        }
        if (i + 1) % width == 0 {
            result.push('\n');
            if (i + 1) % (24 * 8) == 0 {
                result.push('\n');
            }
        }
    }
    result
}

fn nessie() -> Array2<u8> {
    let beastie = arr1(&[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0,
        0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0,
    ]);
    beastie.into_shape((3, 20)).unwrap()
}

fn mask_match(win: &ArrayView2<u8>, mask: &ArrayView2<u8>) -> bool {
    Zip::from(win)
        .and(mask)
        // We only care about the state of w where the mask is nonzero.
        .all(|&w_elem, &m_elem| (m_elem == 0) || (w_elem != 0))
}

fn find_image_locations(haystack: &Array2<u8>, mask: &Array2<u8>) -> Vec<(i32, i32)> {
    let haystack_width = haystack.shape()[1];
    haystack
        .windows((mask.shape()[0], mask.shape()[1]))
        .into_iter()
        .enumerate()
        .filter_map(|(i, w)| {
            if mask_match(&w.view(), &mask.view()) {
                log::debug!("found sea monster at i={}", i);
                let y = i / haystack_width;
                let x = i % haystack_width;
                Some((x as i32, y as i32))
            } else {
                None
            }
        })
        .collect()
}

fn count_ones(m: &Array2<u8>) -> usize {
    m.iter().filter(|&&x| x == 1).count()
}

fn measure_roughness(bitmap: &Array2<u8>, locations: &Vec<(i32, i32)>, mask: &Array2<u8>) -> usize {
    count_ones(bitmap) - (locations.len() * count_ones(mask))
}

fn solve2(tiles: &HashMap<TileId, Tile>, solution: &TileLocationSolution) -> usize {
    let big_bitmap = assemble_big_bitmap(tiles, solution);
    log::debug!("big bitmap is:\n{}", render_bitmap(&big_bitmap));
    let nessie_mask = nessie();
    for rot in [
        Rotation::Zero,
        Rotation::One,
        Rotation::Two,
        Rotation::Three,
    ] {
        for flip in [false, true] {
            let manip = Manipulation { rot, flip };
            let tweaked = manip.on(&big_bitmap);
            let locations = find_image_locations(&tweaked, &nessie_mask);
            log::info!("Part 2: monster locations ({}): {:?}", manip, locations);
            if !locations.is_empty() {
                return measure_roughness(&tweaked, &locations, &nessie_mask);
            }
        }
    }
    panic!("found no sea monsters");
}

fn part2(tiles: &HashMap<TileId, Tile>, solution: &TileLocationSolution) -> Result<(), String> {
    println!("Part 2: roughness is {}", solve2(tiles, solution));
    Ok(())
}

fn run() -> Result<(), String> {
    self_test()?;

    let mut buffer = String::new();
    let tiles: HashMap<TileId, Tile> = match io::stdin().read_to_string(&mut buffer) {
        Ok(_) => read_tiles(&buffer),
        Err(e) => {
            return Err(format!("I/O error: {}", e));
        }
    };
    let solution = part1(&tiles)?;
    part2(&tiles, &solution)?;
    Ok(())
}

fn main() {
    // the env logger is configured with $RUST_LOG.
    // For example RUST_LOG=debug day20
    pretty_env_logger::init();
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {}", err);
            1
        }
    });
}
