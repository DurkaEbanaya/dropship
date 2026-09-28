
# dropship v3 — Linux / openSUSE fork

Native Linux port of [stowmyy/dropship](https://github.com/stowmyy/dropship),
with Wayland/X11, Steam/Proton and Wine process detection, and per-user
nftables and iptables/ip6tables backends.

**[Linux build, install and usage instructions](docs/linux.md)** — includes
openSUSE Tumbleweed dependencies, an installer, RPM packaging and firewall tests.
Linux blocks selected server networks for the user's UDP game traffic rather
than filtering by Windows executable path. Start the Linux binary directly.

The upstream Windows documentation is retained below. Its `.exe` download and
Windows self-updater are not Linux builds.

<!--
```ruby
OW2 // DROPSHIP
```
-->

<table>
 <tr>
  <td>
   <img src="https://github.com/stowmyy/dropship/assets/120167078/55ba7db6-7d37-4eec-b50f-e1e52192009f" />
  </td>
  <td>
   a portable server selector for overwatch
  </td>
  <td>
    <a href="https://github.com/stowmyy/dropship/releases/latest/download/dropship.exe" target="_blank">
     <p></p>
     <pre>download</pre>
    </a>
  </td>
    <td>
    <a href="https://github.com/stowmyy/dropship/releases" target="_blank">
     <p></p>
     <pre>versions</pre>
    </a>
  </td>
 </tr>
</table>

<!--
## notes
- not affiliated with blizzard
- twitter: [@stormyy_ow](https://twitter.com/stormyy_ow/)
-->
 
## how to use
1. download (using the button above) and open dropship with your game closed
3. **block** the servers you **don't** want to play on
4. done! you can close the app now. blocking servers persists until you unblock them. there is also an option to only have these blocks while the app is open

## warnings
> [!IMPORTANT]
> <ul><li>when queuing with other people, the matchmaker may put you on servers you've avoided unless all other members in the party also block them</li><li>practice range does not use the matchmaker. if you'd like to test dropship properly, try custom games</li><li>If you are ever failing to connect to a server, quickly clicking this button will prevent a competitive ban. please report any instance of this on the discord so it can be investigated<br /> <img width="420" height="64" alt="image" src="https://github.com/user-attachments/assets/9045d2f7-d4e0-484b-9f67-25adb8174d60" /></li></ul> 

## faq
question | answer
:-------------------------|:-------------------------
it's not working :( | please write a message in https://discord.gg/QYrF8CVhbC or open an issue here https://github.com/stowmyy/dropship/issues
do i need to keep the app open? | no - blocking servers is permanent (unless you select the option to block only while dropship is open)
how do i uninstall? | click the `disable dropship` button and then delete the `.exe`
does this app install anything? | this app creates a configuration file in `%appdata%/dropship`. the only other changes it makes to your computer are adding an ip filter to windows for blocking game servers, and some temporary files when you update the app. the filter is deleted whenever all servers are unblocked
how do i pin this app to my taskbar? | <img src="https://github.com/user-attachments/assets/a0cf3cf5-4b24-4ee5-b893-b95a73b9e75b" height="90" />
does this app modify the game? | this app does **_not_** modify the game in any way. instead, this app works by adding ip address filters (wfp) just like an ad blocker would
i want a feature | suggestions are welcome in the discord. please don't contribute ai generated code.

## media
| expanded | collapsed |
| -- | -- |
| <img width="1511" height="1060" alt="image" src="https://github.com/user-attachments/assets/df7b7ebc-ecc2-472a-b3ab-4dbf0d3cc6d8" /> | <img width="484" height="1063" alt="image" src="https://github.com/user-attachments/assets/9b246ae8-3112-40ee-b0fa-2c1118197f92" /> |
