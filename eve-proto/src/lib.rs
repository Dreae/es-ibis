#![feature(let_chains)]

pub mod value;
pub mod opcodes;
pub mod decode;
pub mod string_table;

#[cfg(test)]
mod tests {
    pub mod test_data;
}