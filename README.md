
## build web
`wasm-pack build --target web`

`cargo run --example server` NOTE: can use PORT

### gh-page
`git checkout gh-pages`

edit index as needed. copy /pkg files to root. commit and push


## Test

 `cat input.xml |cargo run |rustfmt`
