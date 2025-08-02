# prip

Dumps all files recursively from a MTP device (a camera in most cases).

## Usage

Clone the repo and run with cargo or install as bin `cargo install --path .`

```
Usage: prip [OPTIONS] --output <OUTPUT>

Options:
  -v, --verbose...       Increase logging verbosity
  -q, --quiet...         Decrease logging verbosity
  -o, --output <OUTPUT>  Output directory path
  -d, --delete           Delete files from device after copy
  -h, --help             Print help
  -V, --version          Print version
```


`prip -o ./output_dir/date -d`

`cargo run -- -o ./output_dir/date -d`

## External ependencies:

install libgphoto2 on your system

`sudo apt install libgphoto2-dev libclang-dev`
or
`brew install libgphoto2`

or try to compile the binary and follow compiler errors specific for your system.


