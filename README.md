Demo: https://sbeckeriv.github.io/to_serde/

I use https://transform.tools/json-to-rust-serde for json. I had a need for some xml and was looking for a project.

I put structs that were lines with attributes and data in a mod with the struct name. so if `one` has a `link` with different requirements from `two` they can still use the name link. sub elements live at the same level and could have conflicting names. 

I did my best to detect if something can be missing. if feeds has a number of entry elements i merge them and try to figure out if they can be missing.

## build web
`wasm-pack build --target web`

`cargo run --example server` NOTE: can use PORT

### gh-page
`git checkout gh-pages`

edit index as needed. copy /pkg files to root. commit and push


## Test

 `cat input.xml |cargo run |rustfmt`


## wishlist

rustfmt doenst compile to wasm. i did my best to format it.
a nice webpage. tabs for different formats, helpful text,
options in formatting. 