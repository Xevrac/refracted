using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_GarrisonTransferEntities
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.GarrisonTransferEntities); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.GarrisonTransferEntities)obj;
            //  Serialize OldPlayerId
            s.Write(value.OldPlayerId);
            //  Serialize OldGarrisonId
            s.Write(value.OldGarrisonId);
            //  Serialize NewPlayerId
            s.Write(value.NewPlayerId);
            //  Serialize NewGarrisonId
            s.Write(value.NewGarrisonId);
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize array GarrisonableIds
            Rts.Serialization.Reference.Write(s, value.GarrisonableIds, () =>
            {
                s.WriteVarInt32(value.GarrisonableIds.Length);
                for(int i = 0 ; i < value.GarrisonableIds.Length ; ++i)
                {
                    s.Write(value.GarrisonableIds[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.GarrisonTransferEntities)) as Rts.CnC.Messages.Client.GarrisonTransferEntities;
            //  Deserialize OldPlayerId
            s.Read(out value.OldPlayerId);
            //  Deserialize OldGarrisonId
            s.Read(out value.OldGarrisonId);
            //  Deserialize NewPlayerId
            s.Read(out value.NewPlayerId);
            //  Deserialize NewGarrisonId
            s.Read(out value.NewGarrisonId);
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize array GarrisonableIds
            Rts.Serialization.Reference.Read(s, out value.GarrisonableIds, () =>
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
