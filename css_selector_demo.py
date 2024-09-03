import bs4
from requests import get


URL = "https://novelfull.com/everyone-wants-to-pamper-the-bigshot-researcher-after-her-rebirth.html"

COVER_SELECTOR = ".book > img"

response = get(URL)
soup = bs4.BeautifulSoup(response.text, "html.parser")

with open("test.html", "w") as file:
    file.write(soup.prettify())


cover = soup.select_one(COVER_SELECTOR)

print(cover["src"])