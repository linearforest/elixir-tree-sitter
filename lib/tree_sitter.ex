defmodule TreeSitter do
  use Rustler,
    otp_app: :tree_sitter,
    crate: :elixir_tree_sitter

  @moduledoc """
  Documentation for `TreeSitter`.
  """

  defmodule Node do
    defstruct [
      :id,
      :kind,
      :range,
      :is_named,
      :is_error,
      :is_extra,
      :is_missing,
      children: []
    ]
  end

  defmodule Range do
    defstruct [:start_byte, :end_byte, :start_point, :end_point]
  end

  defmodule Point do
    defstruct [:row, :column]
  end

  defmodule Token do
    defstruct [:kind, :node_type, :value]
  end

  @doc """
  Hello world.

  ## Examples

      iex> TreeSitter.hello()
      :world

  """
  def hello do
    :world
  end

  def parse(_corpus, _language), do: :erlang.nif_error(:nif_not_loaded)
  def lex(_corpus, _language), do: :erlang.nif_error(:nif_not_loaded)
  def to_sexp(_corpus, _language), do: :erlang.nif_error(:nif_not_loaded)
end

defimpl Inspect, for: TreeSitter.Point do
  import Inspect.Algebra

  def inspect(point, opts) do
    concat([point.row |> Inspect.inspect(opts), ":", point.column |> Inspect.inspect(opts)])
  end
end

defimpl Inspect, for: TreeSitter.Token do
  import Inspect.Algebra

  def inspect(token, opts) do
    concat([
      "#T<",
      token.kind,
      ">"
    ])
  end
end

defimpl Inspect, for: TreeSitter.Node do
  import Inspect.Algebra

  def inspect(node, opts) do
    anonymous? = Keyword.get(opts.custom_options, :anonymous, false)
    extra? = Keyword.get(opts.custom_options, :extra, false)

    range =
      concat([
        node.range.start_point |> Inspect.inspect(opts),
        "..",
        node.range.end_point |> Inspect.inspect(opts)
      ])

    kind =
      if node.is_named do
        node.kind
      else
        concat(["\"", node.kind, "\""])
      end

    doc =
      "("
      |> glue("", kind)
      |> glue(range)
      |> group()

    doc =
      case node.children do
        [] ->
          doc

        children ->
          e = empty()

          inner =
            for node <- node.children,
                node.is_named || anonymous? || (node.is_extra && extra?),
                reduce: e do
              ^e -> Inspect.inspect(node, opts)
              acc -> acc |> glue(Inspect.inspect(node, opts))
            end

          doc
          |> glue(inner)
      end

    doc
    |> nest(4)
    |> glue("", ")")
    |> group()
  end
end
