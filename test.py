import furas

with open('./tests/assets/shop.html') as f:
    html = f.read()

model = furas.generate_model(html, {
    "test": "div.col:nth-child(1) > div:nth-child(1) > img:nth-child(1)"
})

print(model)
print(model.group_signature)

res = furas.extract(model, html)
print(res)
