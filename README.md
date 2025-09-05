# norwegian-tin-validator

A robust Norwegian TIN (Tax Identification Number) validator library supporting both D-numbers and F-numbers, including the new TIN format introduced in 2032. Also supports all synthetic test TINs and all test IDs used by Skatteetaten.

## Features

- Validate Norwegian TINs (F-numbers and D-numbers)
- Support for new TIN format (from 2032)
- Handles synthetic test TINs
- Supports all test IDs used by Skatteetaten
- Simple API for integration

## Installation

Add to your `Cargo.toml`:

```toml
norwegian-tin-validator = "0.1"
```

## Usage

### Rust

```rust
use norwegian_tin_validator::NorwegianTin;

fn main() {
    let tin = "01010112345";
    if NorwegianTin::parse(tin).is_ok() {
        println!("Valid Norwegian TIN!");
    } else {
        println!("Invalid TIN.");
    }
}
```

## Documentation

- [New TIN format (2032)](https://skatteetaten.github.io/folkeregisteret-api-dokumentasjon/nytt-fodselsnummer-fra-2032)

## License

Apache-2.0 License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions and issues are welcome! Please open an issue or pull request on [GitHub](https://github.com/kristoffer/norwegian-tin-validator).

## Acknowledgements

Based on official documentation from [Skatteetaten](https://skatteetaten.github.io/folkeregisteret-api-dokumentasjon/nytt-fodselsnummer-fra-2032).
