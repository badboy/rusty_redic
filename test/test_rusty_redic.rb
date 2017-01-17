#!/usr/bin/env ruby

require 'minitest/autorun'
require 'rusty_redic'

#
# Extremely basic unit test for rusty_redic
#
class TestRustyRedic < MiniTest::Test
  def test_new

    assert RustyRedic.new
  end

  def test_call

    redic = RustyRedic.new

    assert_equal "OK", redic.call(["SET", "foo", "bar"])
    assert_equal "bar", redic.call(["GET", "foo"])
  end
end
