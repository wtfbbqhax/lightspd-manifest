# lightspd-manifest
![Docker-Image CI](https://github.com/wtfbbqhax/lightspd-manifest/actions/workflows/rust.yml/badge.svg)

This tool will generate a minimal manifest from the `Talos_LightSPD.tar.gz` from any arbitrary Snort version. The problem it solves is pretty basic, but important. 


## Problem

At the time of this writing, the latest Snort 3 release version is `3.1.56.0`, while the latest Talos LightSPD package contains a compatible set of `policies`, `rules`, `builtins` and `modules` versioned at compatibility boundaries. This makes it quite awkward to figure out what you need to get running.

The following is the directory tree of the lightspd package. The challenge is to identify the slice of this tree which is required to run any particular version of Snort. 


```
lightspd/builtins:
3.0.0.0-0/

lightspd/rules:
3.0.0.0/
3.1.35.0/

lightspd/modules:
3.0.1.0/
3.1.11.0/
3.1.15.0/
3.1.18.0/
3.1.21.1-114/
3.1.26.0/
3.1.35.0/
3.1.44.0/
3.1.7.0/
3.1.9.0/
src/
stubs/

lightspd/policies:
3.0.0-268/
3.0.1-3/
3.0.2-3/
3.0.3-1/
3.0.3-4/
3.1.0.0-0/
common/
```

## How to

Next, lets start carving the lightspd package into minimal sets. Note to follow this, you'll need to have a Snort.org account and your `OINKCODE`.


1. Build
    ```sh
    cargo build
    ```

2. Extract minimal Snort runtime configuration

    ```sh
    # First, download the latest LightSPD package if you haven't already
    curl -LGo Talos_LightSPD.tar.gz \
        "https://www.snort.org/rules/Talos_LightSPD.tar.gz?oinkcode=$OINKCODE"
    
    # Determine your Snort version
    # In my case, I have Snort 3.0.2-6
    snort --dump-version

    # Generate the manifest for your version and architecture.
    # For no particular reason I choose centos-x64, which works for me.
    lightspd-manifest 3.0.2-6 centos-x64 ./Talos_LightSPD.tar.gz > manifest.txt

    # Last, extract the minimum slice
    tar -xf --files-from=manifest.txt Talos_LightSPD.tar.gz
    ```

3. Verify the results


    ```sh
    $ ls -1 lightspd/*

    lightspd/version.txt

    lightspd/builtins:
    3.0.0.0-0/

    lightspd/modules:
    3.0.1.0/
    stubs/

    lightspd/policies:
    3.0.2-3/
    common/

    lightspd/rules:
    3.0.0.0/
    ```


## Credits
* Victor Roemer ([wtfbbqhax](https://www.github.com/wtfbbqhax))
