defmodule TreeSitter do
  use Rustler,
    otp_app: :tree_sitter,
    crate: :elixir_tree_sitter
  @moduledoc """
  Documentation for `TreeSitter`.
  """

  @doc """
  Hello world.

  ## Examples

      iex> TreeSitter.hello()
      :world

  """
  def hello do
    :world
  end

  def add(_a, _b), do: :erlang.nif_error(:nif_not_loaded)
end
