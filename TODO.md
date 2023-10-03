# General
- [x] aniwatch scraper
- [x] manga reader
- [ ] movies ?

## Clap
- [x] make the cli/tui that combines all the apps


## Movie/Tv show

### Needed for video subs
- [ ] find a way to differentiate tv show from movie

### Movie
- [ ] get the movie id from the search (17997 from "/movie/watch-the-girl-next-door-17997")
- [ ] get the episode id from some server in "https://flixhq.to/ajax/movie/episodes/17997"  (data-linkid="5300443")
- [x] use the data link id in "https://flixhq.to/ajax/sources/5300443" to get the provider link (https://rabbitstream.net/embed-4/hjw1u7Ys3g5r?z=)
- [x] reformat the link to "https://rabbitstream.net/ajax/embed-4/getSources?id=hjw1u7Ys3g5r" and get the encrypted video sources and captions
- [ ] decrypt the video sources and play them


#### Tv show
- [x] get the movie id from the search (99826 from "/tv/watch-one-piece-99826")
- [x] use this id in "https://flixhq.to/ajax/v2/tv/seasons/99826" to get all the seasons and season ids (data-id="77971")
- [x] use the season id in "https://flixhq.to/ajax/v2/season/episodes/77971" to get all episodes and episode ids (data-id="1372999" for ep 1)
- [x] use this episode id in "https://flixhq.to/ajax/v2/episode/servers/1372999" to get the server id from some server (data-id="9889051")
- [x] use this id in "https://flixhq.to/ajax/sources/9889051" to get the provider link (https://rabbitstream.net/embed-4/rsZatGlNTN40?z=)
- [x] reformat the link to "https://rabbitstream.net/ajax/embed-4/getSources?id=rsZatGlNTN40" and get the encrypted video sources and captions
- [x] decrypt the video sources and play them



## Manga
- [ ] change site ?
- [ ] next previous buttons


## Anime
- [ ] decrypt it from rust not with the openssl app ?
- [x] remove the recursion when changing to the next ep
- [ ] history
- [ ] select to watch dubbed ?
- [ ] intro skip ?
- [x] remove crash after trying to go past the last ep

### needed for video and subs:
- [x] get the ep id from anime name and number of ep example: (2295 from "https://aniwatch.to/watch/one-piece-100?ep=2295")
- [x] then use this id in "https://aniwatch.to/ajax/v2/episode/servers?episodeId=2295" to get the data-id from some provider ex: (451151)
- [x] then use the data id in "https://aniwatch.to/ajax/v2/episode/sources?id=451151" to get the link (https://megacloud.tv/embed-2/e-1/qwGaKY4erzNn?k=1)
- [x] then use the host (megacloud.tv) and the id (qwGaKY4erzNn) in "https://megacloud.tv/embed-2/ajax/e-1/getSources?id=qwGaKY4erzNn" to get the encrypted video sources and the captions
- [x] decrypt the video sources (idk how)
- [x] play the video from the decrypted url 



