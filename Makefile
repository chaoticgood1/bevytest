text: text
	cd examples && cargo make serve

wasm: w
w:
	cd wasm && cargo build --target wasm32-unknown-unknown