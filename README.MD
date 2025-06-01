<div align="center">
    <img width="256" height="256" src="./.github/images/logo.png" alt="">
</div>

<div align="center">
    <h1>ShadowSniff</h1>
    <p>Lightweight information stealer written in Rust, targeting Windows systems. It interacts directly with the Windows API and avoids unnecessary external dependencies, resulting in a compact, efficient, and low-footprint binary. (PoC. For Educational Purposes only)</p>
    <p>Made in Russia.</p> 
    <div align="center">
        <a href="https://github.com/sqlerrorthing/ShadowSniff/network/members"><img src="https://img.shields.io/github/forks/sqlerrorthing/ShadowSniff?style=for-the-badge&labelColor=%239f0000&color=%23D73332" alt="Forks"></a>
        <a href="https://github.com/sqlerrorthing/ShadowSniff/stargazers"><img src="https://img.shields.io/github/stars/sqlerrorthing/ShadowSniff?style=for-the-badge&labelColor=%239f0000&color=%23D73332" alt="Stargazers"></a>
        <a href="https://github.com/sqlerrorthing/ShadowSniff/issues"><img src="https://img.shields.io/github/issues/sqlerrorthing/ShadowSniff?style=for-the-badge&labelColor=%239f0000&color=%23D73332" alt="Issues"></a>
        <a href="https://github.com/sqlerrorthing/ShadowSniff/blob/master/LICENSE"><img src="https://img.shields.io/github/license/sqlerrorthing/ShadowSniff?style=for-the-badge&labelColor=%239f0000&color=%23D73332" alt="License"></a>
    </div>
</div>

---

_This project is currently under development, and a fully functional version has not been released yet.
Features and stability may change.
Please consider enabling notifications to stay informed about updates and important announcements._

## Features
Despite its lightweight design, ShadowSniff delivers extensive functionality, 
covering all essential data extraction capabilities for Windows targets.

### Log Sending
- [ ] Discord webhook
- [ ] Telegram bot

### Browsers
| Feature      |                                Chromium (<127, see [here](https://github.com/sqlerrorthing/ShadowSniff/blob/master/utils/src/browsers/chromium.rs#L79))                                 |           Gecko            |
|--------------|:---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------:|:--------------------------:|
| Supported    | Amingo, Torch, Kometa, Orbitum, Epic Private, Cent, Vivaldi, **Chromium**, Thorium, **Opera**, **Opera GX**, 7Star, Chrome SxS, **Chrome**, **Edge**, Uran, **Yandex**, **Brave**, Atom | **Firefox**, **Librewolf** |
| AutoFill     |                                                                                            ✅                                                                                            |             ❌              |
| Bookmarks    |                                                                                            ✅                                                                                            |             ❌              |
| Cookies      |                                                                                            ✅                                                                                            |             ✅              |
| Passwords    |                                                                                            ✅                                                                                            |             ❌              |
| Credit cards |                                                                                            ✅                                                                                            |             ❌              |
| Downloads    |                                                                                            ✅                                                                                            |             ❌              |
| History      |                                                                                            ✅                                                                                            |             ✅              |

### Messengers
- [x] Discord
- [x] Telegram sessions

### System
- [x] Process list
- [x] Clipboard
- [x] Screenshot
- [x] System info
- [x] User info

### VPN
- [x] OpenVPN

_... more_

If you'd like the stealer to support additional features, feel free to open an [issue](https://github.com/sqlerrorthing/ShadowSniff/issues/new/choose) here to request the functionality —
or implement it yourself.

## Contributing
Feel free to open issues or submit pull requests — all contributions are welcome,
whether it's a bug fix, a new feature, or just an idea.

⭐ You can also support the project by starring the repository — it's 
a small action that makes a big difference!

Thanks for being part of the project ❤️

## Acknowledgments
This project has been inspired by a variety of open-source infostealers available on GitHub.
Many functions and critical paths were adapted from publicly available repositories.
The goal was to build something original by learning from and evolving existing work.
I’d like to thank everyone whose contributions helped shape this project — with special appreciation to:

- [davimiku](https://github.com/davimiku/json_parser) — for a clean and efficient JSON parser implementation
- [CasualX](https://github.com/CasualX/obfstr) — for compile-time string obfuscation

...and many others whose code, ideas, or techniques helped shape this project —
even if only through a single clever line of Rust.

## License
This library is released under the MIT License. See LICENSE file for more information.

---

## Disclaimer

### Important Notice: This tool is intended for educational purposes only.
This software, known as ShadowSniff, is intended solely for educational and research purposes.
It must never be used for malicious purposes, including (but not limited to) unauthorized access,
data breaches, or any other harmful or illegal activities.

### Usage Responsibility:
By using this tool, you accept full responsibility for your actions.
Any form of misuse is strictly forbidden.
The creator (sqlerrorthing) bears no liability for how the tool is used.
It is your sole duty to ensure that your usage complies with all relevant laws and regulations in your area.

### No Liability:
The creator (sqlerrorthing)
of this tool assumes no responsibility for any damage or legal consequences that may result from its use or misuse.
This includes,
but is not limited to, any direct, indirect, incidental, consequential,
or punitive damages arising from your access to, use of, or inability to use the software.

### No Support:
The creator (sqlerrorthing) will not offer any support, guidance, or assistance in cases involving misuse of this tool.
Any inquiries related to malicious or unauthorized activities will be disregarded.

### Acceptance of Terms:
By using this tool, you acknowledge and accept the terms of this disclaimer.
If you do not agree with these terms, you must refrain from using the software.