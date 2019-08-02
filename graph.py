from __future__ import print_function
#from __future__ import unicode_literals
from __future__ import division
from __future__ import absolute_import

import sys
import io
import lldb
import debugger
import struct
import tempfile
import base64

sys.path.append("/usr/local/lib/python2.7/site-packages")
from graphviz import Digraph

def chunked(size, source):
  for i in range(0, len(source), size):
    yield source[i:i + size]


class Node(object):
  def __init__(self, buf):
    tup = struct.unpack('?xxxx' + ('?xxxx' + 'Q') * 3 + 'Q', buf)
    self.color = tup[0]
    self.parent = None if not tup[1] else tup[2]
    self.left = None if not tup[3] else tup[4]
    self.right = None if not tup[5] else tup[6]
    self.val = tup[7]

  def __repr__(self):
    return "Node(color={}, paren={}, left={}, right={}, val={})".format(self.color, self.parent, self.left, self.right, self.val)


class RBTreeMeta(object):
  def __init__(self, meta):
    root = struct.unpack('?xxxxQ', meta)
    self.root = None if not root[0] else root[1]


class Mem(object):
  def __init__(self, mem):
    inner = mem.GetChildMemberWithName(
        'mmap').GetChildMemberWithName('inner')
    ptr = inner.GetChildMemberWithName('ptr').GetValueAsUnsigned()

    self.len = struct.unpack('Q', bytearray(
        lldb.process.ReadMemory(ptr, 8, lldb.SBError())))[0]
    self.meta = RBTreeMeta(
        bytearray(lldb.process.ReadMemory(ptr + 8, 16, lldb.SBError())))

    self.data = [Node(bytearray(lldb.process.ReadMemory(
        ptr + 24 + offset, 64, lldb.SBError()))) for offset in range(0, self.len * 64, 64)]


class RBTree(object):
  def __init__(self, tree):
    self.capacity = tree.GetChildMemberWithName(
        'capacity').GetValueAsUnsigned()
    self.mem = Mem(tree.GetChildMemberWithName('mem'))

  def toDigramInner(self, g, node):
    if node is None:
      return
    node = self.mem.data[node]
    if node.left is None and node.right is None:
      return
    if node.left is not None:
      left = self.mem.data[node.left]
      g.edge(str(node.val), str(left.val))
      self.toDigramInner(g, node.left)
    else:
      name = "left{}".format(node.val)
      g.node(name, '', shape="point")
      g.edge(str(node.val), name, '')
    if node.right is not None:
      right = self.mem.data[node.right]
      g.edge(str(node.val), str(right.val))
      self.toDigramInner(g, node.right)
    else:
      name = "right{}".format(node.val)
      g.node(name, '', shape="point")
      g.edge(str(node.val), name, '')

  def toDigram(self):
    g = Digraph(format='png')
    g.attr('graph', ordering='out')
    for i in range(self.mem.len):
      node = self.mem.data[i]
      color = "red" if node.color else "black"
      g.node(str(node.val), "{},{}".format(i, node.val), color=color)
    self.toDigramInner(g, self.mem.meta.root)
    return g

def noop():
  pass

def view(tree):
  tree = debugger.unwrap(tree)
  tree = RBTree(tree)
  diag = tree.toDigram()
  temp = tempfile.mkdtemp()
  path = diag.render("tree.png", temp, view=True)
  #with open(path, "rb") as f:
  #  data = base64.b64encode(f.read())

  #document = '<html><body style="overflow: scroll;"><img src="data:image/png;base64,{}" style="zoom: 10"></body></html>'.format(data)
  #debugger.display_html(document, title="Tree", position=2, reveal=False)
