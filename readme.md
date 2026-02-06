# cowboy tower

![banner](https://github.com/ingobeans/cowboy-tower/blob/main/web/banner.png?raw=true)

cowboy tower is an action platformer written in rust hwere you are cowboy!! you need to climb the tower and defeat bosses to reach the top! 

theres bandits, horses, lasers, lava, wall climbing, monsters and mooooreeeeeeee!!!!!!!

![cowboy](https://github.com/ingobeans/cowboy-tower/blob/main/web/cowboy.png?raw=true)

## about

all assets and code made by me. no ai (slop) here!

project made entirely for Hackclub's [flavortown](https://flavortown.hackclub.com/)!

the project is written in rust, but also uses GLSL shaders. as of writing this the project is almost nearing 4000 lines !

![madeinrustbutton.png](./web/madeinrustbutton.png)
![madeinaseprite.png](./web/madeinaseprite.png)
![madeintiledbutton.png](./web/madeintiledbutton.png)

## building

the project is made in rust, so youll need that installed.

#### standalone
run with
```bash
cargo run
```

#### web builds

to build for web, youll first need to compile for WASM, then serve the webpage.

for example, if you were using `basic-http-server` to serve, you could do:
```bash
cargo build --release --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/release/cowboy-tower.wasm web/ && basic-http-server web/
```