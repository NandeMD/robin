import bs4
from requests import get


URL = "https://novelfull.com/martial-peak/chapter-2-breaking-through-the-wall-and-not-looking-back.html"

COVER_SELECTOR = "#chapter-content > p"

response = get(URL)
soup = bs4.BeautifulSoup(response.text, "html.parser")

with open("test.html", "w") as file:
    file.write(soup.prettify())


cover = soup.select(COVER_SELECTOR)

print(cover[2].text)
