# hb.cgi

"The Handlebars but as a CGI-app". Because it's fun. And helps Web to stay Smol.

## Usage

Just do GET with url-encoded

- `t` as a Handlebars template URI
- `d` as a JSON data URI

Also you can POST the same stuff either form-encoded or in JSON body with forementioned `t`+`d` params in both cases.

## Examples

[Makefile](Makefile) runs the `lighttpd` as a server which then serves all the stuff from the [dev/](dev/) folder. One can just open http://localhost:8000/ and click some buttons. The code here is pretty self-explanatory, so you're welcome to play with!
