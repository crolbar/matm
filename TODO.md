# General
- [x] aniwatch scraper
- [ ] manga reader
- [ ] movies ?


## Clap
- [ ] make the cli/tui that combines all the apps


## Anime
### optimization!!!
- [ ] decrypt it from rust not with openssl
- [ ] combine ep and ep_ids into an hashmamp
- [ ] make it run fasterr!!!
- [ ] remove the infinite loop when changing to the next ep if there even is one ?

### needed for video and subs:
- [x] get the ep id from anime name and number of ep example: (2295 from "https://aniwatch.to/watch/one-piece-100?ep=2295")
- [x] then use this id in "https://aniwatch.to/ajax/v2/episode/servers?episodeId=2295" to get the data-id from some provider ex: (451151)
- [x] then use the data id in "https://aniwatch.to/ajax/v2/episode/sources?id=451151" to get the link (https://megacloud.tv/embed-2/e-1/qwGaKY4erzNn?k=1)
- [x] then use the host (megacloud.tv) and the id (qwGaKY4erzNn) in "https://megacloud.tv/embed-2/ajax/e-1/getSources?id=qwGaKY4erzNn" to get the encrypted video sources and the captions
- [x] decrypt the video sources (idk how)
- [x] play the video from the decrypted url 



