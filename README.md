# clutter
A CLI Texture Packer written in rust that can pack a bunch of images optimally based on [crunch-rs](https://github.com/ChevyRay/crunch-rs). Supports many formats as well including json, binary and [ron](https://github.com/ron-rs/ron).

### Usage
This CLI Texture Packer is based around the config file, which you would have to create in order to pack the images.
Example content of the config file with ron format:
```ron
// Json is also supported
PackerConfig(
    name: "gem-collections",
    output_path: "out",
    // In Json format, output_type is a string.
    output_type: Json,
    folders: [
        "images/common",
        "images/rare",
        "images/legendary"
    ],
    options: PackerConfigOptions(
        max_size: 4096,
        show_extension: false,
        // we wouldn't want this to be true for now, it's not working :/
        rotation: false
    )
)
```
Then, in the CLI usage:

`clutter <FORMAT>-path --input <INPUT>`

Example:

`clutter ron-path --input packer-config.ron`

### Binary Format
```
[UInt32] - Count (Use it in for loops below)
  L [String] - Name
    [UInt32] - X
    [UInt32] - Y
    [UInt32] - Width
    [UInt32] - Height
```
