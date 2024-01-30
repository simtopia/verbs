pragma solidity 0.8.10;

contract Contract {
  int256 private _x;

  event ValueUpdated(int256 old_value, int256 new_value);

  constructor(int256 x) {
    _x = x;
  }

  function getValue() external view returns (int256) {
    return _x;
  }

  function setValue(int256 x) public {
    int256 old_x = _x;
    _x = x;
    emit ValueUpdated(old_x, x);
  }
}
