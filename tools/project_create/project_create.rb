#############################################################################
#
# Script to auto-create habitat core plan projects in a specified depot
#
# Usage: ruby project_create.rb <core-plans-dir> <projects-url> <auth-token>
#
# The projects-url should be in this form:
# http://app.acceptance.habitat.sh/v1/projects
#
#############################################################################

require 'erb'
require 'net/http'
require 'uri'
require 'json'

if ARGV.length != 3
  puts "Usage: pcreate <core-plans-dir> <projects-url> <auth-token>"
  exit
end

source_dir = ARGV[0]
url = ARGV[1]
auth_token = ARGV[2]

template = File.read('project.erb')
renderer = ERB.new(template)

uri = URI.parse(url)
http = Net::HTTP.new(uri.host, uri.port)

Dir.chdir source_dir
Dir.open '.' do |root|
  root.each do |f|
      if f.index(".") != 0 && File.directory?(f)
        plan = f.to_s
        plan_path = File.join(f, 'plan.sh')

        if File.exists?(plan_path)
          puts "Creating project for #{plan}"
          req = Net::HTTP::Post.new(uri, {"Authorization" => "Bearer #{auth_token}"})
          req.body = renderer.result(binding)
          res = http.request(req)
          puts "Response: #{res}"
        else
          puts "WARNING: plan.sh not found at #{plan_path} - skipping"
        end
      end
  end
end
