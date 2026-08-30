using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestRepair
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestRepair); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestRepair)obj;
            //  Serialize RepairingPlayerId
            s.Write(value.RepairingPlayerId);
            //  Serialize array RepairingUnitIds
            Rts.Serialization.Reference.Write(s, value.RepairingUnitIds, () =>
            {
                s.WriteVarInt32(value.RepairingUnitIds.Length);
                for(int i = 0 ; i < value.RepairingUnitIds.Length ; ++i)
                {
                    s.Write(value.RepairingUnitIds[i]);
                }
            });
            //  Serialize DamagedPlayerId
            s.Write(value.DamagedPlayerId);
            //  Serialize DamagedUnitId
            s.Write(value.DamagedUnitId);
            //  Serialize ModifierFlags
            s.WriteEnum(value.ModifierFlags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestRepair)) as Rts.CnC.Messages.Client.RequestRepair;
            //  Deserialize RepairingPlayerId
            s.Read(out value.RepairingPlayerId);
            //  Deserialize array RepairingUnitIds
            Rts.Serialization.Reference.Read(s, out value.RepairingUnitIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize DamagedPlayerId
            s.Read(out value.DamagedPlayerId);
            //  Deserialize DamagedUnitId
            s.Read(out value.DamagedUnitId);
            //  Deserialize ModifierFlags
            s.ReadEnum(out value.ModifierFlags);

            return value;
        }
        
    }
}
