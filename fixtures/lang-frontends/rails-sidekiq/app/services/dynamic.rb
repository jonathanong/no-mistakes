class Dynamic
  def call(name)
    kls = const_get(name)
    kls.perform_async
    name.constantize.perform_later
  end
end
