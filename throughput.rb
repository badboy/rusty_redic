# Compare performance of redic with rusty_redic
#
# Run with
#
#   $ bundle exec rake thermite:build
#   $ bundle exec throughput.rb
#

require "rubygems"
require "benchmark"
require "rusty_redic"
require "redic"

$redic = Redic.new
$rusty_redic = RustyRedic.new

# make sure both are connected
$redic.call("PING")
$rusty_redic.call(["PING"])

$redic.call(:flushdb)

def without_gc
  GC.start
  GC.disable
  yield
ensure
  GC.enable
end

def pipeline(b,num,size,title,cmd)
  commands = size.times.map { cmd }

  x = without_gc {
    b.report("redic:       %2dx #{title} pipeline, #{num} times" % size) {
      num.times {
        commands.each { |cmd| $redic.queue(*cmd) }
        $redic.commit
      }
    }
  }

  y = without_gc {
    b.report("rusty_redic: %2dx #{title} pipeline, #{num} times" % size) {
      num.times {
        commands.each { |cmd| $rusty_redic.queue(cmd) }
        $rusty_redic.commit
      }
    }
  }

  puts "%.1fx" % [1 / (y.real / x.real)]
end

Benchmark.bm(50) do |b|
  pipeline(b,10000, 1, "SET", %w(set foo bar))
  pipeline(b,10000,10, "SET", %w(set foo bar))
  puts

  pipeline(b,10000, 1, "GET", %w(get foo))
  pipeline(b,10000,10, "GET", %w(get foo))
  puts

  pipeline(b,10000, 1, "LPUSH", %w(lpush list fooz))
  pipeline(b,10000,10, "LPUSH", %w(lpush list fooz))
  puts

  pipeline(b,1000, 1, "LRANGE(100)", %w(lrange list 0  99))
  puts

  pipeline(b,1000, 1, "LRANGE(1000)", %w(lrange list 0 999))
  puts

  # Clean up...
  redic.call(:flushdb)
end
