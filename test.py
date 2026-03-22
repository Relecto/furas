import furas

with open('./tests/assets/shop.html') as f:
    html = f.read()

model = furas.generate_model(html, {
    "test": "div.col:nth-child(1) > div:nth-child(1) > img:nth-child(1)"
})
