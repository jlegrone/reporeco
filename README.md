# reporeco

This is a proof of concept for generating Github repository recommendations based on [Github stars](https://help.github.com/en/articles/about-stars) using [recoreco](https://github.com/sscdotopen/recoreco).

## Install

```cli
cargo install --git https://github.com/jlegrone/reporeco
```

## Usage

Gather repository metadata from Github:

```cli
GITHUB_API_TOKEN=<my-token> reporeco gather
```

Train reporeco from the gathered data:

```cli
reporeco train
```

Generate recommendations:

```cli
GITHUB_API_TOKEN=<my-token> reporeco recommend kubernetes/kubernetes
```
