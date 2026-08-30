using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ServerHello
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ServerHello); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ServerHello)obj;
            //  Serialize array PlayerHandle
            Rts.Serialization.Reference.Write(s, value.PlayerHandle, () =>
            {
                s.WriteVarInt32(value.PlayerHandle.Length);
                for(int i = 0 ; i < value.PlayerHandle.Length ; ++i)
                {
                    s.Write(value.PlayerHandle[i]);
                }
            });
            //  Serialize array PlayerBlazeId
            Rts.Serialization.Reference.Write(s, value.PlayerBlazeId, () =>
            {
                s.WriteVarInt32(value.PlayerBlazeId.Length);
                for(int i = 0 ; i < value.PlayerBlazeId.Length ; ++i)
                {
                    s.Write(value.PlayerBlazeId[i]);
                }
            });
            //  Serialize array PlayerType
            Rts.Serialization.Reference.Write(s, value.PlayerType, () =>
            {
                s.WriteVarInt32(value.PlayerType.Length);
                for(int i = 0 ; i < value.PlayerType.Length ; ++i)
                {
                    s.Write(value.PlayerType[i]);
                }
            });
            //  Serialize array AllegianceLevel
            Rts.Serialization.Reference.Write(s, value.AllegianceLevel, () =>
            {
                s.WriteVarInt32(value.AllegianceLevel.Length);
                for(int i = 0 ; i < value.AllegianceLevel.Length ; ++i)
                {
                    s.Write(value.AllegianceLevel[i]);
                }
            });
            //  Serialize array Faction
            Rts.Serialization.Reference.Write(s, value.Faction, () =>
            {
                s.WriteVarInt32(value.Faction.Length);
                for(int i = 0 ; i < value.Faction.Length ; ++i)
                {
                    s.Write(value.Faction[i]);
                }
            });
            //  Serialize array General
            Rts.Serialization.Reference.Write(s, value.General, () =>
            {
                s.WriteVarInt32(value.General.Length);
                for(int i = 0 ; i < value.General.Length ; ++i)
                {
                    s.Write(value.General[i]);
                }
            });
            //  Serialize array TeamId
            Rts.Serialization.Reference.Write(s, value.TeamId, () =>
            {
                s.WriteVarInt32(value.TeamId.Length);
                for(int i = 0 ; i < value.TeamId.Length ; ++i)
                {
                    s.Write(value.TeamId[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ServerHello)) as Rts.CnC.Messages.Client.ServerHello;
            //  Deserialize array PlayerHandle
            Rts.Serialization.Reference.Read(s, out value.PlayerHandle, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize array PlayerBlazeId
            Rts.Serialization.Reference.Read(s, out value.PlayerBlazeId, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt64[] tmp = new System.UInt64[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize array PlayerType
            Rts.Serialization.Reference.Read(s, out value.PlayerType, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize array AllegianceLevel
            Rts.Serialization.Reference.Read(s, out value.AllegianceLevel, () =>
            {
                int length = s.ReadVarInt32();
                System.Single[] tmp = new System.Single[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize array Faction
            Rts.Serialization.Reference.Read(s, out value.Faction, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize array General
            Rts.Serialization.Reference.Read(s, out value.General, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize array TeamId
            Rts.Serialization.Reference.Read(s, out value.TeamId, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
