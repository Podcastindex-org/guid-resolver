# guid-resolver
Resolves podcast guids to feed urls.

This is a podcast guid resolver.  It takes a CSV file of guids and feed urls and loads them in a hashmap.  When a GET
request is sent, it's `host` header is parsed to find the first part.  That substring is then used as the key lookup 
on the hashmap.  If a url value is found for that key, it is returned as a plain/text HTTP response body.

Send a GET to:

  917393e3-1b1e-5cef-ace4-edaa54e1f810.guid.podcastindex.org
  
...and get back:

  http://mp3s.nashownotes.com/pc20rss.xml