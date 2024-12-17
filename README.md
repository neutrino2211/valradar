# VALRADAR

> [!WARNING]  
> This tool is in a beta state, although little changes are to be expected proceed with caution.

<p align="center">
    <img src="docs/icon.png" alt="valradar icon" width="200"/>
</p>

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![License][license-shield]][license-url]

https://github.com/user-attachments/assets/a6b5b149-839f-402a-811d-e049ff01fd73

![Demo video](docs/demo.mp4)

Valradar is an offensive security tool designed to enable security researchers to quickly look for certain values en-masse using regex.
<br/>
<br/>
The following types of uses are currently supported:

- [x] Webpages
- [ ] Executables (planned)

### Usage

```sh
Usage: valradar --site=STRING --pattern=STRING [flags]

Search for patterns and strings over a website's footprint

Flags:
  -h, --help                    Show context-sensitive help.

https://github.com/user-attachments/assets/d646d03c-c5a8-4cdf-b7db-48240a03dee2


  -s, --site=STRING             The website to scan
  -d, --depth=1                 How deep to search
  -c, --concurrency=10          How many coroutines to use
  -p, --pattern=STRING          The regex pattern to try matching
      --use-headless-browser    Use a headless chrome browser to fetch the webpages
```

Scanning a website can be done as follows:

```sh
go run . --site https://facebook.com --pattern "(M|m)eta"
```

To improve results on websites with a lot of javascript, use a headless browser. __NOTE: THIS REQUIRES PLAYWRIGHT DEPENDENCIES TO BE INSTALLED__

```sh
go run . --site https://facebook.com --pattern "(M|m)eta" --use-headless-browser
```

To improve performance, increase concurrency but be aware that this value is dependent on the performance of the computer running it.

```sh
go run . --site https://facebook.com --pattern "(M|m)eta" -c 25 --use-headless-browser
```

To search deeper into a website, set the depth option to a number you desire. The behaviour is for the program to scan further into reconvered links for each increase of the depth number

```sh
go run . --site https://facebook.com --pattern "(M|m)eta" -c 25 --use-headless-browser -d 20
```

[contributors-shield]: https://img.shields.io/github/contributors/neutrino2211/valradar?style=for-the-badge
[contributors-url]: https://github.com/neutrino2211/valradar/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/neutrino2211/valradar?style=for-the-badge
[forks-url]: https://github.com/neutrino2211/valradar/network/members
[stars-shield]: https://img.shields.io/github/stars/neutrino2211/valradar?style=for-the-badge
[stars-url]: https://github.com/neutrino2211/valradar/stargazers
[issues-shield]: https://img.shields.io/github/issues/neutrino2211/valradar?style=for-the-badge
[issues-url]: https://github.com/neutrino2211/valradar/issues
[license-shield]: https://img.shields.io/github/license/neutrino2211/valradar?style=for-the-badge
[license-url]: https://github.com/neutrino2211/valradar/blob/master/LICENSE
