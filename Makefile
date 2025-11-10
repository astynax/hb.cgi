root := $(shell pwd)

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
