# Molyuu Controller

Molyuu Controller is used to manage and register available sessions in the system. It supports X.org and Wayland
environments. All sessions are accessed through registered aliases. This project is used to replace the complex session
selection script in SteamOS and facilitate users to change the desktop environment by themselves.

## Usage

1. Switch default Login/Display Manager (Support lightdm, sddm)

```shell
$ molyuuctl login set-manager lightdm
```

2. Login to DE via Login/Display Manager

```shell
$ sudo molyuuctl login now
```

3. Start a session (Requires execution via session launcher)

```shell
$ molyuuctl session start <Session Register Name>
```

4. Set a session to start one-shot next time

```shell
$ molyuuctl session set-oneshot <Session Register Name>
```

For more useage, please see help page of the program

```
$ molyuuctl --help
```