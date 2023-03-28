# Onsite
## A url-by-url sitemap generator

### Usage:

```
onsite [options]
        -h |  --help : print this help message
        -c | --clean : remove all urls from the sitemap
        -f |  --file : specify the sitemap file (default: sitemap.xml)

    -a | --add <url> : add a url to the sitemap (gets escaped)
    --lastmod <date> : set the url's lastmod property
 --changefreq <freq> : set the url's changefreq property
    --priority <pri> : set the url's priority property

 -r | --remove <url> : remove a url from the sitemap

            --to-url : transform a filepath into a url (requires --root)
       --root <root> : the root (plus protocol) for the url
    --old-root <old> : a prefix to strip from the filepath
         --clean-url : removes `index.*` from the end of the filepath
```

See [the sitemap official website](https://sitemaps.org/protocol) for more info.