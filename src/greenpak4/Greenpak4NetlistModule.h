/***********************************************************************************************************************
 * Copyright (C) 2016 Andrew Zonenberg and contributors                                                                *
 *                                                                                                                     *
 * This program is free software; you can redistribute it and/or modify it under the terms of the GNU Lesser General   *
 * Public License as published by the Free Software Foundation; either version 2.1 of the License, or (at your option) *
 * any later version.                                                                                                  *
 *                                                                                                                     *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied  *
 * warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Lesser General Public License for     *
 * more details.                                                                                                       *
 *                                                                                                                     *
 * You should have received a copy of the GNU Lesser General Public License along with this program; if not, you may   *
 * find one here:                                                                                                      *
 * https://www.gnu.org/licenses/old-licenses/lgpl-2.1.txt                                                              *
 * or you may search the http://www.gnu.org website for the version 2.1 license, or you may write to the Free Software *
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301, USA                                      *
 **********************************************************************************************************************/

#ifndef Greenpak4NetlistModule_h
#define Greenpak4NetlistModule_h

#include <string>
#include <vector>
#include <json-c/json.h>

class Greenpak4Netlist;
class Greenpak4NetlistPort;
class Greenpak4NetlistNode;

//only for RTTI support and naming
class Greenpak4NetlistEntity
{
public:
	Greenpak4NetlistEntity(std::string name = "")
	: m_name(name)
	{}

	virtual ~Greenpak4NetlistEntity();
	
	std::string m_name;
};

class Greenpak4NetlistModule;

//A single primitive cell in the netlist
//TODO: move to separate header
class Greenpak4NetlistCell : public Greenpak4NetlistEntity
{
public:
	Greenpak4NetlistCell(Greenpak4NetlistModule* module)
	: m_parent(module)
	{ m_parnode = NULL; }
	virtual ~Greenpak4NetlistCell();
	
	bool HasParameter(std::string att)
	{ return m_parameters.find(att) != m_parameters.end(); }
	
	//Indicates whether the cell is an I/O buffer
	bool IsIOB()
	{ return (m_type == "GP_IBUF") || (m_type == "GP_IOBUF") || (m_type == "GP_OBUF") || (m_type == "GP_OBUFT"); }
	
	//Called by Greenpak4PAREngine::InitialPlacement_core
	void FindLOC();
	
	std::string GetLOC()
	{ return m_loc; }
	
	bool HasLOC()
	{ return (m_loc != ""); }
	
	//Clear the LOC constraint if it's determined to be bogus (TODO: better handling of this situation)
	void ClearLOC()
	{ m_loc = ""; }
	
	///Module name
	std::string m_type;
	
	std::map<std::string, std::string> m_parameters;
	std::map<std::string, std::string> m_attributes;
	
	typedef std::vector<Greenpak4NetlistNode*> cellnet;
	
	/**
		@brief Map of connections to the cell
		
		connections[portname] = {bit2, bit1, bit0}
	 */
	std::map<std::string, cellnet > m_connections;
	
	PARGraphNode* m_parnode;
	
	//Parent module of the cell, not the module we're an instance of
	Greenpak4NetlistModule* m_parent;	
	
protected:
	std::string m_loc;
};

#include "Greenpak4NetlistNode.h"

/**
	@brief A single module in a Greenpak4Netlist
 */
class Greenpak4NetlistModule
{
public:
	Greenpak4NetlistModule(Greenpak4Netlist* parent, std::string name, json_object* object);
	virtual ~Greenpak4NetlistModule();
	
	Greenpak4NetlistNode* GetNode(int32_t netnum);
	
	std::string GetName()
	{ return m_name; }
	
	std::map<std::string, std::string> m_attributes;

	typedef std::map<std::string, Greenpak4NetlistPort*> portmap;
	typedef std::map<std::string, Greenpak4NetlistCell*> cellmap;
	typedef std::map<std::string, Greenpak4NetlistNode*> netmap;
	
	portmap::iterator port_begin()
	{ return m_ports.begin(); }
	
	portmap::iterator port_end()
	{ return m_ports.end(); }
	
	cellmap::iterator cell_begin()
	{ return m_cells.begin(); }
	
	cellmap::iterator cell_end()
	{ return m_cells.end(); }
	
	netmap::iterator net_begin()
	{ return m_nets.begin(); }
	
	netmap::iterator net_end()
	{ return m_nets.end(); }
	
	bool HasNet(std::string name)
	{ return (m_nets.find(name) != m_nets.end()); }
	
	Greenpak4NetlistNode* GetNet(std::string name)
	{ return m_nets[name]; }
	
	Greenpak4NetlistPort* GetPort(std::string name)
	{ return m_ports[name]; }
	
	Greenpak4Netlist* GetNetlist()
	{ return m_parent; }
	
	//Add an extra cell (used by make_graphs to add inferred ACMPs etc)
	void AddCell(Greenpak4NetlistCell* cell)
	{ m_cells[cell->m_name] = cell; }
		
	//Add an extra net (used by make_graphs to add inferred VREFs etc)
	void AddNet(Greenpak4NetlistNode* net)
	{ m_nets[net->m_name] = net; }
		
protected:
	Greenpak4Netlist* m_parent;
	
	///Internal power/ground nets
	Greenpak4NetlistNode* m_vdd;
	Greenpak4NetlistNode* m_vss;
	
	void CreatePowerNets();
	
	std::string m_name;
	
	void LoadAttributes(json_object* object);
	void LoadNetName(std::string name, json_object* object);
	void LoadNetAttributes(Greenpak4NetlistNode* net, json_object* object);
	void LoadCell(std::string name, json_object* object);
	void LoadCellAttributes(Greenpak4NetlistCell* cell, json_object* object);
	void LoadCellParameters(Greenpak4NetlistCell* cell, json_object* object);
	void LoadCellConnections(Greenpak4NetlistCell* cell, json_object* object);
	
	std::map<int32_t, Greenpak4NetlistNode*> m_nodes;
	portmap m_ports;
	netmap m_nets;
	cellmap m_cells;
};

#endif
