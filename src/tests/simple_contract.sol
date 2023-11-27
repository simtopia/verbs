// SPDX-License-Identifier: BUSL-1.1
pragma solidity 0.8.10;

contract Contract {
  int256 private _x;

  constructor(int256 x) {
    _x = x;
  }

  function getValue() external view returns (int256) {
    return _x;
  }

  function setValue(int256 x) public {
    _x = x;
  }
}
