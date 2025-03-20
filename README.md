<a name="readme-top"></a>
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]




<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/QEDProtocol/doge-light-client">
  <img width="256" height="256" src="https://github.com/QEDProtocol/doge-light-client/raw/main/static/sol-doge-logo.png?raw=true">
  </a>

  <h3 align="center">DLC: Doge Light Client</h3>

  <p align="center">
A fully featured light client for Dogecoin implemented in Rust 🦀, designed for implementing efficient trustless bridges between Dogecoin and the rest of the blockchain universe 🌍 (especially Solana～).
  </p>
  <p align="center">Made with ❤️ by <a href="https://x.com/cmpeq" target="_blank">Carter Feldman</a> at <a href="https://QEDProtocol.com" target="_blank">QED</a></p>
</div>

## Features
* Supports correct calculation of block difficulty for Mainnet, Testnet, and Regtest
* Supports AuxPow/Non-AuxPow blocks for Mainnet and Testnet
* Maintains a constant-memory merkle tree of all blocks processed
* Fully zerocopy for on-chain usage




## License
Copyright (C) 2025 Zero Knowledge Labs Limited, QED Protocol

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

Additional terms under GNU AGPL version 3 section 7:

As permitted by section 7(b) of the GNU Affero General Public License, 
you must retain the following attribution notice in all copies or 
substantial portions of the software:

"This software was created by QED (https://qedprotocol.com)
with contributions from Carter Feldman (https://x.com/cmpeq)."