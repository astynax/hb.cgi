root := $(shell pwd)
image := "hb-cgi-builder"

.PHONY: server
server: dev/hb.cgi dev/lighttpd.conf
	lighttpd -D -f dev/lighttpd.conf

dev/lighttpd.conf: dev/lighttpd.conf.template
	m4 -D xROOT="$(root)" $< > $@

dev/hb.cgi: target/debug/hb-cgi
	@true

target/debug/hb-cgi: Cargo.toml src/main.rs
	cargo build

.PHONY: clean
clean:
	rm -v -f dev/lighttpd.conf dev/hb.cgi

.PHONY:
builder:
	docker build -t $(image) .

.PHONY: release
release: Cargo.toml src/main.rs
	docker run --rm --user "$(id -u)":"$(id -g)" \
		-v "$(root)":/usr/src/myapp:ro \
		-v "$(root)/release/target":/work:rw \
		-w /usr/src/myapp \
		$(image) \
		cargo build --release --target-dir=/work
