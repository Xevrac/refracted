using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_SelectionUpdate
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.SelectionUpdate); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.SelectionUpdate)obj;
            //  Serialize array PlayerIds
            Rts.Serialization.Reference.Write(s, value.PlayerIds, () =>
            {
                s.WriteVarInt32(value.PlayerIds.Length);
                for(int i = 0 ; i < value.PlayerIds.Length ; ++i)
                {
                    s.Write(value.PlayerIds[i]);
                }
            });
            //  Serialize array UnitIds
            Rts.Serialization.Reference.Write(s, value.UnitIds, () =>
            {
                s.WriteVarInt32(value.UnitIds.Length);
                for(int i = 0 ; i < value.UnitIds.Length ; ++i)
                {
                    s.Write(value.UnitIds[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.SelectionUpdate)) as Rts.CnC.Messages.Client.SelectionUpdate;
            //  Deserialize array PlayerIds
            Rts.Serialization.Reference.Read(s, out value.PlayerIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize array UnitIds
            Rts.Serialization.Reference.Read(s, out value.UnitIds, () =>
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
