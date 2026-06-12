# Targets
.PHONY: build-web serve-web clean

# Directories
WASM_OUT = wasm-out
WEB_SRC = web

# Build the WebAssembly version
build-web:
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --out-dir $(WASM_OUT) --target web target/wasm32-unknown-unknown/release/tightpack.wasm
	mkdir -p $(WASM_OUT)
	cp $(WEB_SRC)/index.html $(WASM_OUT)/
	@echo "Web build finished. Run 'make serve-web' to start a local server."

# Serve the web build using Python 3
serve-web:
	cd $(WASM_OUT) && python3 -m http.server 8000

# Clean the web build directory
clean:
	rm -rf $(WASM_OUT)

run-web: build-web serve-web