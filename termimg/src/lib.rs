pub mod graphic;
pub mod kitty;
pub mod apc;
pub mod utils;
pub mod sixel;
pub mod iterm;
pub mod term;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
