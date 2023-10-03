defmodule TreeSitterTest do
  use ExUnit.Case
  doctest TreeSitter

  test "greets the world" do
    assert TreeSitter.hello() == :world
  end

  test "works" do
    assert TreeSitter.add(1, 2) == 3
  end
end
