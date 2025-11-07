.PHONY: server
server: dev/hb.cgi dev/lighttpd.conf
	lighttpd -D -f dev/lighttpd.conf

dev/lighttpd.conf: dev/lighttpd.conf.template
	sed "s|\$$PWD|$$(pwd)|" $< > $@

dev/hb.cgi: target/debug/hb-cgi
	cp $< $@

target/debug/hb-cgi: Cargo.toml src/main.rs
	cargo build
