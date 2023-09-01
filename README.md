# > *aggy*

Experiment.

## Design

Forwards in, first there's `aggy_nextjs`, a simple Hacker News clone web app with users, posts, comments, the usual shebang. 
It talks to `aggy_api` for most of the user and post related features like auth, submissions and finding posts for the front page. 
For the heirarchical comment threads `epigram_api` steps into the picture. 
As far as it's concerned, the world's a tree of crypto-signed messages and all replies and even post contents are stored and queried from this service.
`doface_api`(TBD) does the same for reactions, it's world only crypto-signed short utf-8 strings.
This *will* be how upvotes/flags will be implemented for `aggy_api` posts. 
Yes, crypto-signed. 
Pubkeys and everything. 
`epigram` and `doface` will support different kinds of pubkeys and signitures (think multiformats) but they'll primarily be of secp256k1 Schnorr as [Nostr](https://github.com/nostr-protocol/nostr) events will the primary input/ingest vector of the system.
To this end exists `qtrunk_api`, a dumb nostr relay implementation for this usecase.

All in all, `epigram`, `doface` and even `aggy` itself can be considered secondary cache layers for Nostr even if they expose separate REST interfaces and are generally agnostic of it.

Speaking of the practicals, all the services make use of PostgreSQL for presistence (subject to change) and Redis for caching/event streaming. 
What's more, even though the different services are written in different crates, use separate databases and have careful abstractions for any cross talk, they're currently run in a combined single process that serves each under different prefixes. 
That's what the `aggynfrens_api` crate's for.

## Rationale

There's no resoundingly good rationale for secondary cache layers over Nostr, especially for these usecases. 

## *Goals*

- [ ] Support javascript disabled browsers in `aggy_nextjs`
    - [ ] Allow Nostr free usage path
- [ ] Support [NIP-07](https://github.com/nostr-protocol/nips/blob/master/07.md) in `aggy_nextjs`
