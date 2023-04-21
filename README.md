# cluttered
[![crates.io][crates-svg]][crates-link] [![github][github-svg]][github-link]


A CLI Texture Packer written in rust that can pack a bunch of images optimally based on [crunch-rs](https://github.com/ChevyRay/crunch-rs). Supports many formats as well including json, binary and [ron](https://github.com/ron-rs/ron).

It can be used for games with Godot, Monogame or a bunch more that needed to optimize the memory usage.

### Usage
This CLI Texture Packer is based around the config file, which you would have to create in order to pack the images.
Example content of the config file with toml format:
```toml
name = "gem_collections"
output_path = "samples/out"
output_type = "Json"
template_path = "samples/xml-template.xml"
folders = [
    "samples/images/common",
    "samples/images/rare",
    "samples/images/legendary",
    "samples/images/rect"
]

[image_options]
output_extension = "Png"
max_size = 4096
show_extension = false

[features]
nine_patch = true
```
Then, in the CLI usage:

`cluttered config --input <INPUT>`

Example:

`cluttered config --input packer-config.toml`

Alternatively, we can use the manual way, which we can use the argument called `pack`


`cluttered pack --input <[INPUT]> --output <OUTPUT>`

Example:

`cluttered pack --input images/legendary images/rare --output out --type json`

## Pack Arguments

|name          |description|
|--------------|-----------|
|--type        |Specify the output type.
|--name        |Specify the name of the output.
|--templatepath|Specify the template path for the custom template output.

## Binary Format
```
[String] - Name
[UInt32] - Count (Use it in for loops below)
  L [String] - Name
    [UInt32] - X
    [UInt32] - Y
    [UInt32] - Width
    [UInt32] - Height
    [Bool] - HasNinePatch (if nine_patch is enabled)
    [UInt32] - X (if nine_patch is enabled and has a file)
    [UInt32] - Y (if nine_patch is enabled and has a file)
    [UInt32] - W (if nine_patch is enabled and has a file)
    [UInt32] - H (if nine_patch is enabled and has a file)
```

[crates-svg]: https://img.shields.io/crates/v/cluttered.svg
[github-svg]: https://img.shields.io/github/v/release/Terria-K/cluttered
[github-link]: https://github.com/Terria-K/cluttered/releases
[crates-link]: https://crates.io/crates/cluttered

## Template
We also support custom template with [handlebars](https://handlebarsjs.com/guide/block-helpers.html) write your own custom template to support your favourite format or make a new one.

Example usage on xml format:
```xml
<TextureAtlas imagePath="{{atlas.sheet_path}}">
  {{#each atlas.frames as frame}}
  <Texture 
      name="{{@key}}" 
      x="{{this.x}}" y="{{this.y}}" 
      width="{{this.width}}" height="{{this.height}}"
      {{#if this.nine_patch}}
      nx="{{this.nine_patch.x}}" ny="{{this.nine_patch.y}}"
      nw="{{this.nine_patch.w}}" nh="{{this.nine_patch.h}}"
      {{/if}}
    />
  {{/each}}
</TextureAtlas>
```

## Global Variables
|Name         |Type       |
|-------------|-----------|
|config       |Config
|atlas        |PackerAtlas


### Config

|Name                |Type       |
|--------------------|-----------|
|name                |String
|output_path         |String
|output_type         |OutputType
|folders             |String[]
|allow_normal_output |bool,
|template_path       |String | String[]?
|options             |ImageOptions
|features            |Features

### OutputType (Strings on Json)
|Name         |
|-------------|
|Json         |
|Binary       |
|Ron          |
|Template     |


### ImageOptions
|Name             |Type |
|-----------------|-----|
|max_size         |int  |
|output_extension |OutputExtensionType
|show_extension   |bool


### OutputExtensionType (Strings on Json)
| Name |
|------|
|Png   |
|Qoi   |
|Jpg   |


### Features
|Name          |Type       |
|--------------|-----------|
|nine_patch    |bool

### PackerAtlas
|Name          |Type       |
|--------------|-----------|
|sheet_path    |String
|frames        |HashMap<String, TextureData>

### TextureData
|Name          |Type       |
|--------------|-----------|
|x             |int
|y             |int
|width         |int
|height        |int
|nine_patch    |Rect?

### Rect
|Name          |Type       |
|--------------|-----------|
|x             |int
|y             |int
|w             |int
|h             |int
