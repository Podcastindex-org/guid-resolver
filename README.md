# guid-resolver
Resolves podcast guids to feed urls.

This is a web front-end to a Redis DB.  It takes a podcast guid as a subdomain and resolves it to a feed url, giving back the url
as a plain text response.

Send a GET to:

  917393e3-1b1e-5cef-ace4-edaa54e1f810.guid.podcastindex.org
  
...and get back:

  http://mp3s.nashownotes.com/pc20rss.xml