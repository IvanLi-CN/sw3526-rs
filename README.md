# SW3526 Driver Documentation

This is a driver for the SW3526, built on the [embedded-hal](https://crates.io/crates/embedded-hal) framework. It supports both `async` and `sync` features and is designed for `no_std` environments.

## Usage

To add the SW3526 driver to your project, run the following command:

```shell
# sync
cargo add sw3526

# async
cargo add sw3526 --features async
```

For an example of how to use this driver in your project, you can check out [power-desk](https://github.com/IvanLi-CN/power-desk?tab=readme-ov-file).

## License

This project is licensed under the [MIT](LICENSE) license.
