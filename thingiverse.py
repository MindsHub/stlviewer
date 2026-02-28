import re
import requests
import urllib3
from collections import OrderedDict
import json

HEADERS = OrderedDict({
    #"Host": urllib3.util.SKIP_HEADER,
    "User-Agent": urllib3.util.SKIP_HEADER,
    "Accept-Encoding": urllib3.util.SKIP_HEADER,
    "Accept": None,
    "Connection": None
})


def getRequestNoFingerprint(url: str, headers=None):
    headers2 = HEADERS.copy()
    headers2.update(headers or {})

    s = requests.Session()
    s.headers = {}
    r = s.get(url, headers=headers2, timeout=5)
    return r

def getAuthBearer():
    data = getRequestNoFingerprint("https://cdn.thingiverse.com/site/js/app.bundle.js").text
    return re.compile(r"\"([a-z0-9]{32})\"").search(data).group(1)

class ThingiverseExtractor:
    def __init__(self):
        self.auth = "56edfc79ecf25922b98202dd79a291aa" # getAuthBearer()

    def _jsonRequest(self, url: str):
        return getRequestNoFingerprint(url, headers={"Authorization": f"Bearer {self.auth}"}).json()

    # name is e.g. "italy"
    def getGroupId(self, name: str):
        return self._jsonRequest(f"https://www.thingiverse.com/api/groups/{name}")["id"]

    def getGroupThings(self, group_id: int):
        page = 1
        things = []
        while True:
            new = self._jsonRequest(f"https://www.thingiverse.com/api/groups/{group_id}/things?page={page}&per_page=50")
            if len(new) == 0:
                break
            page += 1
            things.append(new)
        return things

print(getAuthBearer())
exit()
e = ThingiverseExtractor()

#print(e._jsonRequest("https://www.thingiverse.com/api/things/6847795/files"))
#print(len(e._jsonRequest("https://www.thingiverse.com/api/groups/184/things?page=3&per_page=50")))
gid = e.getGroupId("italy")
things = e.getGroupThings(gid)
with open("a.json", "w") as f:
    json.dump(things, f, indent=4)