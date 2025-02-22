pub fn coordinate_to_string(pos: (usize, usize)) -> String {
    let file = (b'a' + pos.1 as u8) as char;
    let rank = 8 - pos.0;
    format!("{}{}", file, rank)
}
// old method; not used
pub fn parse_coordinate(coord: &str) -> Option<(usize, usize)> {
    if coord.len() != 2 {
        return None;
    }

    let file = coord.chars().nth(0)?.to_ascii_lowercase();
    let rank = coord.chars().nth(1)?.to_digit(10)?;

    if !('a'..='h').contains(&file) || !(1..=8).contains(&rank) {
        return None;
    }

    let file_idx = (file as u8 - b'a') as usize;
    let rank_idx = 8 - rank as usize;

    Some((rank_idx, file_idx))
}
