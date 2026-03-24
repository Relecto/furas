# Furas

Fuzzy HTML data extraction library for Python, written in Rust.

## Fuzzy?

The most common approach of extracting a piece of data from HTML is using CSS selectors or tree traversal code to locate a tag that contains data of itnerest. This has a couple of drawbacks:

1. Writing selectors or scripts for locating tags is time-consuming and tedious.
2. Even minor changes on the live website to the tag of interest or general document structure can break the script.
3. There are cases in which a tag cannot be identified by a CSS selector alone.

Furas takes a different approach to the problem. 
Instead of locating a tag by CSS selector or other rigid means, furas compares all tags in the document to an expected *signature*, and picks the most similar one.

So, to create a scraper with Furas, you need to:

1. Get page HTML you want to scrape and exact CSS selectors of the tags you want to find.
2. Furas computes signatures of the tags and creates a *Model*. 
3. Save the model somewhere. Now, you can use the Model to scrape other instances of the same page.
