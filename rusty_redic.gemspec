# -*- coding: utf-8 -*-
require 'English'

Gem::Specification.new do |s|
  s.name        = 'rusty_redic'
  s.version     = '0.2.0'
  s.summary     = 'Lightweight Redis Client, in Rust'
  s.description = 'A lightweight Redis Client, written in Rust, as minimal as Redic'

  s.authors     = ['Jan-Erik Rediger']
  s.email       = 'janerik@fnordig.de'
  s.homepage    = 'https://github.com/badboy/rusty_redic'
  s.license     = 'MIT'

  s.extensions    = %w(Rakefile)
  s.files         = `git ls-files`.split($OUTPUT_RECORD_SEPARATOR)
  s.require_paths = %w(lib)
  s.test_files    = %w(test/test_rusty_redic.rb)

  s.add_runtime_dependency 'thermite', '~> 0'
  s.add_development_dependency 'minitest', '~> 5.8'
end
