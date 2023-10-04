defmodule TreeSitterTest do
  use ExUnit.Case
  doctest TreeSitter

  test "greets the world" do
    assert TreeSitter.hello() == :world
  end

  test "works" do
    assert TreeSitter.add(1, 2) == 3
  end

  describe "parse/2" do
    test "javascript" do
      assert {:ok, %TreeSitter.Node{kind: "program", children: [_]}} =
               TreeSitter.parse("1 + 2", :javascript)
    end

    test "html" do
      assert {:ok, %TreeSitter.Node{kind: "fragment", children: [_]}} =
               TreeSitter.parse("<html></html>", :html)
    end

    test "css" do
      assert {:ok, %TreeSitter.Node{kind: "stylesheet", children: [_]}} =
               TreeSitter.parse("body {}", :css)
    end

    test "liquid" do
      assert {:ok, %TreeSitter.Node{kind: "stylesheet", children: [_]}} =
               TreeSitter.parse("{a | b}", :liquid)
    end
  end

  test "to_sexp/2" do
    assert TreeSitter.to_sexp(
             "<html></html>",
             :html
           ) == {:ok, "(fragment (element (start_tag (tag_name)) (end_tag (tag_name))))"}
  end
end
