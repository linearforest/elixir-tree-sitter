defmodule TreeSitterTest do
  alias TreeSitter.Token
  use ExUnit.Case
  doctest TreeSitter

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

    test "liquid_template" do
      assert {:ok,
              %{
                kind: "template",
                children: [%{kind: "output_directive"}]
              }} =
               TreeSitter.parse("{{a | b}}", :liquid_template)
    end
  end

  describe "to_tokens/2" do
    test "javascript" do
      assert TreeSitter.to_tokens("1 + 2", :javascript) ==
               {:ok,
                [
                  %Token{
                    node_type: :named,
                    value: "1",
                    kind: "number"
                  },
                  %Token{node_type: :anonymous, value: "+", kind: "+"},
                  %Token{
                    node_type: :named,
                    value: "2",
                    kind: "number"
                  }
                ]}
    end
  end

  describe "parse_embedded/3" do
    test "works" do
      assert {:ok,
              %{
                html: %{kind: "fragment"},
                liquid: %{kind: "program"},
                liquid_template: %{kind: "template"}
              }} =
               TreeSitter.parse_embedded(
                 "{% if true %}<span>a</span>{% endif %}",
                 :liquid_template,
                 """
                 ((content) @injection.content
                  (#set! injection.language "html"))

                 ((code) @injection.content
                  (#set! injection.language "liquid"))
                 """
               )
    end

    test "parses comments correctly" do
      assert {:ok,
              %{
                html: %{kind: "fragment"},
                liquid: %{
                  kind: "program",
                  children: [
                    %{kind: "identifier"},
                    %{kind: "comment"},
                    %{kind: "identifier"}
                  ]
                },
                liquid_template: %{kind: "template"}
              }} =
               TreeSitter.parse_embedded(
                 "a {{ x }} b {{ # comment }} c {{ y }} d",
                 :liquid_template,
                 """
                 ((content) @injection.content
                  (#set! injection.language "html"))

                 ((code) @injection.content
                  (#set! injection.language "liquid"))
                 """
               )
    end
  end

  test "to_sexp/2" do
    assert TreeSitter.to_sexp(
             "<html></html>",
             :html
           ) == {:ok, "(fragment (element (start_tag (tag_name)) (end_tag (tag_name))))"}
  end
end
