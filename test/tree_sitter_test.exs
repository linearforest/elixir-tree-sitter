defmodule TreeSitterTest do
  use ExUnit.Case
  doctest TreeSitter

  test "greets the world" do
    assert TreeSitter.hello() == :world
  end

  test "works" do
    assert TreeSitter.add(1, 2) == 3
  end

  test "parse javascript" do
    TreeSitter.parse(
      """
      console.log(1 + 2)
      """,
      :javascript
    )
    |> IO.inspect()

    assert TreeSitter.parse(
             """
             console.log(1 + 2)
             """,
             :javascript
           ) == {:ok, %{}}
  end

  test "parse html" do
    assert TreeSitter.parse(
             """
             <html><p>hello</p></html>
             """,
             :html
           ) == {:ok, %{}}
  end

  test "to_sexp/2" do
    assert TreeSitter.to_sexp(
             "<html></html>",
             :html
           ) == {:ok, "(fragment (element (start_tag (tag_name)) (end_tag (tag_name))))"}
  end

  test "parse css" do
    assert TreeSitter.parse(
             """
             body {
               color: red;
             }
             """,
             :css
           ) == {:ok, %{}}
  end

  test "parse liquid" do
    assert TreeSitter.parse(
             """
             <span class="visually-hidden">{{ price | money }}</span>
             """,
             :html
           ) == {:ok, %{}}
  end
end
