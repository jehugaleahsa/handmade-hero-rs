# handmade-hero-rs

Casey Muratori's handmade hero - but in Rust! Because I like hard-mode.

## Things got too crazy!

I wasn't sure whether I'd actually keep watching [Casey](https://caseymuratori.com/about)'s [game programming series](https://mollyrocket.com/#handmade), but I find I am learning something new in every episode.

Things got a bit too awesome, with Casey's code accepting controller and keyboard input, then DirectSound, and I realized I couldn't just watch anymore - I needed to get coding! Being a masochist, I am following along in Rust instead of C/C++.

## About me - Why am I doing this?

I've been coding for over 20 years, so me saying I am seeing something new isn't something to balk at. To elaborate further, this isn't even my first time learning about "the game loop". I bought quite a few books on game programming back in my early days (*and never read most of them*). I recall the painful drudgery of reading an entire chapter to just spit out a black window. Then the author settles on a basic "bricks breaker" game. It was too boring - it focused too much on teaching C++, which I knew very well from college, and not enough on the gaming side of things. I kept buying books hoping one would hit the sweet spot, but I gave up on game programming after a year or two working professionally.

I started learning Rust almost 5 years ago. I did the classic *"read the entire [Rust book](https://doc.rust-lang.org/stable/book/) and then never write any code"* thing. Years later I did it again. Years later I did it again... *again*, but the 3rd time I actually tried to write some code.

Then about 6 months ago I revisited my little side project in Rust and really tried to finish it up and ended up kicking off a much more ambitious project in the process. I've knocked out some pretty substantial code in Rust at this point. But, when you work in Java every day for work, it's hard to stay motivated. Which sucks, because I find lower-level programming much more satisfying. It might relate back to my early days of working C++ and nerding out over all the dumb minutiae.

When I saw Casey getting into some real fun Windows API code, I realized I should start following along on my own. I coincidentally have also been trying to learn some CUDA programming, so had a taste of C/C++ coding again after a while. You know what? Coding in C/C++ is still annoying.

Meanwhile, Rust and its ecosystem are amazing. Someone already did the heavy lifting of exposing Win32 APIs to Rust: [windows-rs](https://github.com/microsoft/windows-rs). Does this lose some of the benefits Casey talks about in regard to keeping things simple? Sure. Am I going to have more fun this way? Absolutely. Will it be a challenge to translate what he's talking about from C to Rust? Sure. Will I end up learning more that way? Absolutely!

## What about all the game assets and stuff?

Obviously, I don't want to post anything in this repo that I am not licensed to share. This repo isn't for *you*, it's just so I don't lose any progress I make, and to help me track what I did from lesson-to-lesson. Consider [supporting Casey](https://mollyrocket.com/#handmade) - getting access to his source code and game assets is pretty cheap and probably well-worth it if you're serious.

If I get to a point where I am using his assets, I will be marking those assets in the `.gitignore`, at which point the software won't work for people just cloning the repo. I will add instruction steps for folks if that time comes. But I will say right now I am not all that interested in repeating what Casey builds verbatim and will most likely experiment with what he's doing rather than build a game clone. Again, not for *you*!

I decided to make this project public in the spirit of sharing. I don't make enough of my GitHub projects public anymore, and this one feels like it's just for fun.

## How are you writing code?

I am not Casey - I do not like emacs. I can `vi`, but I am not productive in it. At first, I was using JetBrain's [RustRover](https://www.jetbrains.com/rust/). Since then, [Zed](https://zed.dev/) went v1.0.0, so I've been trying to use that - somewhat successfully. I still do quite a lot of `cargo` on the command-line, as well as `git`, etc. I have a PowerShell script that I try to run before checking in; I have warnings turned all the way up, and the PS script lints, checks formatting, runs all tests, etc.

## Age of AI (2026-09-26)

At work, I use AI to pretty much do all my coding. The speed is addictive, but half of my job is just reading and reviewing code. It's a constant struggle to stay in control ("hands on the wheel"). This project is now my chance to keep my coding skills sharp. The goal is to use AI as little as possible, except maybe to ask general questions. I want to think through designing solutions and write all the code by hand.
