#[macro_use]
extern crate error_chain;

pub mod error {
    error_chain!{}
}

pub mod build;

#[cfg(test)]
mod tests {
}
