use mvutils::utils::Bytecode;

pub trait PARSER<T> {
    fn parse(b: Bytecode) -> T;
}