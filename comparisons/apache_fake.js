var http = require("http");

http.createServer(function(request, response) {
	response.writeHead(200, {
		"Content-Type": "text/html",
		"Server": "Apache/2.2.22 (Ubuntu)",
		"Last-Modified": "Thu, 05 May 2011 11:46:42 GMT",
		"ETag": "\"501b29-b1-4a285ed47404a\"",
		"Accept-Ranges": "bytes",
		"Content-Length": "177",
		"Vary": "Accept-Encoding",
		"X-Pad": "avoid browser bug",
	});

	response.end("<html><body><h1>It works!</h1>\n<p>This is the default web page for this server.</p>\n<p>The web server software is running but no content has been added, yet.</p>\n</body></html>\n");
}).listen(8001);
